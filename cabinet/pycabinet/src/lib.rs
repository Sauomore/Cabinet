use pyo3::prelude::*;
use pyo3::types::PyList;
use std::path::PathBuf;

use cabinet_core::{Config, Memory, Precision, QueryOpts, QueryResult};

/// PyO3 封装：Python Memory 类
#[pyclass(name = "Memory")]
struct PyMemory {
    inner: Memory,
}

#[pymethods]
impl PyMemory {
    /// 初始化 Memory
    #[new]
    #[pyo3(signature = (path, precision="light", pos_threshold=50, max_context=4096))]
    fn new(path: String, precision: &str, pos_threshold: u32, max_context: usize) -> PyResult<Self> {
        let p = Precision::from_str(precision)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("precision 必须是 'light'/'hybrid'/'precise'"))?;
        let config = Config::new(PathBuf::from(path))
            .precision(p)
            .pos_threshold(pos_threshold)
            .working_memory_size(max_context);
        let mem = Memory::open(config)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(PyMemory { inner: mem })
    }

    /// 插入文本
    fn insert(&mut self, text: String) -> PyResult<u64> {
        self.inner
            .insert(&text)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// 批量插入
    #[pyo3(signature = (texts, show_progress=false))]
    fn insert_batch(&mut self, texts: &Bound<'_, PyList>, show_progress: bool) -> PyResult<Vec<u64>> {
        let mut vec = Vec::with_capacity(texts.len());
        for (i, item) in texts.iter().enumerate() {
            let text: String = item.extract()?;
            vec.push(text);
            if show_progress && (i + 1) % 100 == 0 {
                println!("已插入 {}/{} 条", i + 1, texts.len());
            }
        }
        self.inner
            .insert_batch(&vec)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// 检索
    #[pyo3(signature = (query, top_k=5))]
    fn query(&mut self, query: String, top_k: usize) -> PyResult<Vec<PyQueryResult>> {
        let opts = QueryOpts::new().top_k(top_k);
        let results = self.inner
            .query(&query, opts)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    /// 解码单个查询结果
    fn decode(&self, result: &PyQueryResult) -> PyResult<Option<String>> {
        let qr = QueryResult {
            hsh: result.hsh,
            doc_id: result.doc_id,
            position: result.position,
            match_level: result.match_level,
            score: result.score,
            text: None,
        };
        Ok(self.inner.decode(&qr))
    }

    /// 快照备份
    fn snapshot(&self, dst: String) -> PyResult<()> {
        self.inner
            .snapshot(&PathBuf::from(dst))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// 关闭（同步数据）
    fn close(&mut self) -> PyResult<()> {
        self.inner
            .close()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

/// Python 查询结果对象
#[pyclass(name = "QueryResult")]
#[derive(Clone)]
struct PyQueryResult {
    #[pyo3(get)]
    hsh: u32,
    #[pyo3(get)]
    doc_id: u64,
    #[pyo3(get)]
    position: u32,
    #[pyo3(get)]
    match_level: u8,
    #[pyo3(get)]
    score: f32,
    #[pyo3(get)]
    text: Option<String>,
}

impl From<QueryResult> for PyQueryResult {
    fn from(r: QueryResult) -> Self {
        PyQueryResult {
            hsh: r.hsh,
            doc_id: r.doc_id,
            position: r.position,
            match_level: r.match_level,
            score: r.score,
            text: r.text,
        }
    }
}

#[pymethods]
impl PyQueryResult {
    fn __repr__(&self) -> String {
        format!(
            "QueryResult(hsh=0x{:05X}, doc_id={}, match_level={}, score={:.3})",
            self.hsh, self.doc_id, self.match_level, self.score
        )
    }
}

/// PyO3 模块定义
#[pymodule]
fn pycabinet(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMemory>()?;
    m.add_class::<PyQueryResult>()?;
    Ok(())
}
