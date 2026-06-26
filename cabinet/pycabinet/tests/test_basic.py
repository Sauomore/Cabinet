import pycabinet
import tempfile
import os

def test_basic():
    tmpdir = tempfile.mkdtemp()
    db_path = os.path.join(tmpdir, "test_python.db")

    mem = pycabinet.Memory(db_path, precision="light", pos_threshold=50, max_context=4096)

    # 插入
    doc_id = mem.insert("用户明天下午3点开会，准备PPT。")
    assert doc_id == 1

    doc_id2 = mem.insert("用户喜欢听管弦乐。")
    assert doc_id2 == 2

    # 查询
    results = mem.query("会议准备", top_k=5)
    assert len(results) > 0

    for r in results:
        assert r.score > 0
        assert r.doc_id > 0
        assert r.match_level >= 1
        assert r.match_level <= 4

    # 批量插入
    texts = [f"这是第{i}条测试文档。" for i in range(100)]
    ids = mem.insert_batch(texts, show_progress=False)
    assert len(ids) == 100

    # 快照
    snap_path = os.path.join(tmpdir, "snap.db")
    mem.snapshot(snap_path)
    assert os.path.exists(snap_path)

    mem.close()

    import shutil
    shutil.rmtree(tmpdir)
    print("✓ Python binding tests passed")

if __name__ == "__main__":
    test_basic()
