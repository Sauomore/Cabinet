# -*- coding: utf-8 -*-
"""文档解析模块：支持 PDF、Word、Excel、txt、md、csv 等常见格式。

安装额外依赖：
    pip install cabinet-hsh[docs]

或单独安装：
    pip install pdfplumber python-docx openpyxl

使用示例：
    from cabinet_hsh import document_parser as dp

    # 自动检测格式
    paragraphs = dp.parse_document("./report.pdf")
    for p in paragraphs:
        print(p)

    # 批量插入到记忆库
    mem = cabinet_hsh.Memory(path="./agent.db")
    for text in paragraphs:
        if len(text) > 10:  # 过滤空行
            mem.insert(text)
"""

from __future__ import annotations

import csv
import os
import re
from pathlib import Path
from typing import List


def parse_document(filepath: str) -> List[str]:
    """解析常见文档，返回非空段落/文本行列表。

    Args:
        filepath: 文件路径。

    Returns:
        段落/文本行列表（已去除首尾空白）。

    Raises:
        ValueError: 不支持的文件类型。
        ImportError: 缺少对应的解析依赖（如 pdfplumber、python-docx 等）。
    """
    ext = Path(filepath).suffix.lower()
    parsers = {
        ".pdf": _parse_pdf,
        ".docx": _parse_docx,
        ".doc": _parse_docx,
        ".txt": _parse_text,
        ".md": _parse_text,
        ".rst": _parse_text,
        ".xlsx": _parse_xlsx,
        ".xls": _parse_xlsx,
        ".csv": _parse_csv,
    }
    parser = parsers.get(ext)
    if not parser:
        raise ValueError(
            f"不支持的文件类型: {ext}。支持: {', '.join(parsers.keys())}"
        )
    return parser(filepath)


def _parse_pdf(filepath: str) -> List[str]:
    try:
        import pdfplumber
    except ImportError as exc:
        raise ImportError(
            "解析 PDF 需要 pdfplumber。请安装：pip install pdfplumber"
        ) from exc

    paragraphs = []
    with pdfplumber.open(filepath) as pdf:
        for page in pdf.pages:
            text = page.extract_text()
            if text:
                paragraphs.extend(
                    p.strip() for p in text.split("\n") if p.strip()
                )
    return paragraphs


def _parse_docx(filepath: str) -> List[str]:
    try:
        import docx
    except ImportError as exc:
        raise ImportError(
            "解析 Word 需要 python-docx。请安装：pip install python-docx"
        ) from exc

    doc = docx.Document(filepath)
    paragraphs = [p.text.strip() for p in doc.paragraphs if p.text.strip()]
    return paragraphs


def _parse_text(filepath: str) -> List[str]:
    with open(filepath, "r", encoding="utf-8") as f:
        text = f.read()
    paragraphs = [p.strip() for p in text.split("\n") if p.strip()]
    return paragraphs


def _parse_xlsx(filepath: str) -> List[str]:
    try:
        import openpyxl
    except ImportError as exc:
        raise ImportError(
            "解析 Excel 需要 openpyxl。请安装：pip install openpyxl"
        ) from exc

    wb = openpyxl.load_workbook(filepath, read_only=True, data_only=True)
    paragraphs = []
    for sheet in wb.worksheets:
        for row in sheet.iter_rows(values_only=True):
            for cell in row:
                if cell and isinstance(cell, str):
                    cell_text = cell.strip()
                    if cell_text:
                        paragraphs.append(cell_text)
    return paragraphs


def _parse_csv(filepath: str) -> List[str]:
    paragraphs = []
    with open(filepath, "r", encoding="utf-8", newline="") as f:
        reader = csv.reader(f)
        for row in reader:
            for cell in row:
                if cell and cell.strip():
                    paragraphs.append(cell.strip())
    return paragraphs


def batch_insert_from_file(memory, filepath: str, min_length: int = 10) -> int:
    """从文档解析并批量插入到记忆库。

    Args:
        memory: cabinet_hsh.Memory 实例。
        filepath: 文档路径。
        min_length: 最小段落长度（过滤过短的无意义行）。

    Returns:
        成功插入的段落数。

    Example:
        >>> mem = cabinet_hsh.Memory(path="./agent.db")
        >>> count = batch_insert_from_file(mem, "./report.pdf")
        >>> print(f"已插入 {count} 段")
    """
    paragraphs = parse_document(filepath)
    texts = [p for p in paragraphs if len(p) >= min_length]
    if not texts:
        return 0
    memory.insert_batch(texts, show_progress=True)
    return len(texts)
