#!/usr/bin/env python
"""快速测试 pycabinet 安装是否成功"""
import sys

def test_import():
    print("[测试] 导入 pycabinet...")
    import pycabinet
    print("  ✓ 导入成功")
    return pycabinet

def test_memory():
    print("[测试] 创建 Memory 实例...")
    import pycabinet
    import tempfile
    import os

    tmpdir = tempfile.mkdtemp()
    db_path = os.path.join(tmpdir, "test.db")

    mem = pycabinet.Memory(db_path, precision="light", pos_threshold=50, max_context=4096)
    print("  ✓ 创建成功")

    print("[测试] 插入文档...")
    doc_id = mem.insert("用户明天下午3点开会，准备PPT。")
    print(f"  ✓ 插入 doc_id={doc_id}")

    print("[测试] 查询文档...")
    results = mem.query("会议准备", top_k=5)
    print(f"  ✓ 查询返回 {len(results)} 条结果")

    for r in results:
        print(f"    [{r.score:.2f}] doc_id={r.doc_id} match_level={r.match_level}")

    print("[测试] 关闭...")
    mem.close()
    print("  ✓ 关闭成功")

    # 清理
    import shutil
    shutil.rmtree(tmpdir)
    print("  ✓ 临时文件清理")

def test_batch():
    print("[测试] 批量插入...")
    import pycabinet
    import tempfile
    import os

    tmpdir = tempfile.mkdtemp()
    mem = pycabinet.Memory(os.path.join(tmpdir, "batch_test.db"))

    texts = [f"这是第{i}条测试文档内容。" for i in range(100)]
    ids = mem.insert_batch(texts)
    print(f"  ✓ 批量插入 {len(ids)} 条文档")

    mem.close()

    import shutil
    shutil.rmtree(tmpdir)

def main():
    print("=" * 50)
    print("   pycabinet 安装测试")
    print("=" * 50)
    print()

    try:
        test_import()
        test_memory()
        test_batch()
        print()
        print("=" * 50)
        print("   ✓ 全部测试通过！")
        print("=" * 50)
        return 0
    except Exception as e:
        print()
        print(f"[错误] 测试失败: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(main())
