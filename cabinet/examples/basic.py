# 使用 pycabinet 的示例 Python 脚本
import pycabinet

# 初始化记忆库
mem = pycabinet.Memory(
    path="./agent_memory.db",
    precision="light",
    pos_threshold=50,
    max_context=4096,
)

# 插入一些示例文档
samples = [
    "用户明天下午3点开会，准备PPT。",
    "用户喜欢听管弦乐。",
    "5号楼邻居有梯子，平时放在车库。",
    "上周 user_456 借了梯子给 3 号楼，周三还的。",
    "社区公告：本周六下午有垃圾分类宣传活动。",
    "邻居互助：谁有多余的打印机墨盒？",
]

for text in samples:
    doc_id = mem.insert(text)
    print(f"[插入] doc_id={doc_id}: {text}")

print()

# 查询示例
queries = [
    "会议准备",
    "借梯子",
    "社区活动",
    "打印机",
]

for q in queries:
    results = mem.query(q, top_k=3)
    print(f"[查询] \"{q}\" → {len(results)} 条结果")
    for r in results:
        level_name = {4: "精确", 3: "同簇", 2: "同类", 1: "关联"}.get(r.match_level, "未知")
        print(f"  [{level_name}] score={r.score:.3f} doc_id={r.doc_id}")
        if r.match_level >= 3:
            text = mem.decode(r)
            print(f"    → {text}")
    print()

mem.close()
print("[完成] 示例运行结束，数据保存在 ./agent_memory.db")
