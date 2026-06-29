# -*- coding: utf-8 -*-
"""上下文解码模块：根据检索结果返回词的前/后/整句/整段等上下文。

使用示例：
    import cabinet_hsh
    from cabinet_hsh.context_decoder import decode_context

    mem = cabinet_hsh.Memory(path="./agent.db")
    results = mem.query("会议准备", top_k=5)

    for r in results:
        # 返回整句话
        text = decode_context(mem, r, mode="sentence")
        print(f"整句: {text}")

        # 返回前后 3 个词
        text = decode_context(mem, r, mode="window", window_size=3)
        print(f"前后词: {text}")

        # 返回整段
        text = decode_context(mem, r, mode="paragraph")
        print(f"整段: {text}")
"""

from __future__ import annotations

import re
from typing import Optional


def _split_words(text: str) -> list[str]:
    """切分词（优先用 jieba，否则用正则 fallback）。"""
    try:
        import jieba
        return list(jieba.cut(text))
    except ImportError:
        # 备用：按中文/英文标点+空白切分
        pattern = r"[\s\n,.，。！？；：""''（）《》【】\/\-]+"
        return [w for w in re.split(pattern, text) if w.strip()]


def _split_sentences(text: str) -> list[str]:
    """用中文/英文标点切分句子，保留标点。"""
    # 按句子结束标点分割，但保留标点
    raw = re.split(r"([。！？；…;!\?])", text)
    sentences = []
    for i in range(0, len(raw) - 1, 2):
        sent = raw[i] + (raw[i + 1] if i + 1 < len(raw) else "")
        if sent.strip():
            sentences.append(sent.strip())
    if len(raw) % 2 == 1:
        last = raw[-1].strip()
        if last:
            sentences.append(last)
    return sentences


def _find_sentence_index(words: list[str], pos: int) -> tuple[int, int]:
    """根据词索引找到句子起始和结束位置。

    Returns:
        (sentence_start, sentence_end) 词索引范围。
    """
    sentence_start = 0
    for i in range(pos, -1, -1):
        if words[i] in ("\n", "\r", "。", "！", "？", "；", "…", ".", "!", "?", ";"):
            sentence_start = i + 1
            break

    sentence_end = len(words)
    for i in range(pos, len(words)):
        if words[i] in ("\n", "\r", "。", "！", "？", "；", "…", ".", "!", "?", ";"):
            sentence_end = i + 1
            break

    return sentence_start, sentence_end


def _find_sentence_idx_by_pos(sentences: list[str], words: list[str], pos: int) -> int:
    """根据词索引定位是第几个句子。"""
    word_count = 0
    for idx, sent in enumerate(sentences):
        try:
            import jieba
            sent_words = list(jieba.cut(sent))
        except ImportError:
            sent_words = [w for w in re.split(r"[\s\n,.，。！？；：]+", sent) if w.strip()]
        if word_count + len(sent_words) > pos:
            return idx
        word_count += len(sent_words)
    return max(0, len(sentences) - 1)


def decode_context(
    memory,
    result,
    mode: str = "paragraph",
    window_size: int = 2,
) -> Optional[str]:
    """根据检索结果返回上下文。

    Args:
        memory: cabinet_hsh.Memory 实例。
        result: QueryResult 实例。
        mode: 上下文模式
            - "paragraph"（默认）: 返回整段文档。
            - "sentence": 返回包含匹配词的整句话。
            - "window": 返回前后 ``window_size`` 个词（含匹配词本身）。
            - "before": 返回匹配词前 ``window_size`` 个词。
            - "after": 返回匹配词后 ``window_size`` 个词。
            - "window_sent": 返回前后 ``window_size`` 句话。
        window_size: 窗口大小（词数或句子数）。

    Returns:
        上下文文本，或 ``None``（无文本缓存时）。

    Note:
        Python 层的分词使用 jieba（如果安装）或正则 fallback，与 Rust 核心的
        jieba-rs 分词结果可能略有差异。position 偏移通常在 ±1 个词以内。
    """
    text = memory.decode(result)
    if not text:
        return None

    if mode == "paragraph":
        return text

    words = _split_words(text)
    if not words:
        return text

    pos = min(result.position, len(words) - 1)

    if mode == "sentence":
        start, end = _find_sentence_index(words, pos)
        return "".join(words[start:end])

    elif mode == "window":
        start = max(0, pos - window_size)
        end = min(len(words), pos + window_size + 1)
        return "".join(words[start:end])

    elif mode == "before":
        start = max(0, pos - window_size)
        return "".join(words[start:pos])

    elif mode == "after":
        end = min(len(words), pos + window_size + 1)
        return "".join(words[pos + 1:end])

    elif mode == "window_sent":
        sentences = _split_sentences(text)
        if not sentences:
            return text
        sent_idx = _find_sentence_idx_by_pos(sentences, words, pos)
        start = max(0, sent_idx - window_size)
        end = min(len(sentences), sent_idx + window_size + 1)
        return "，".join(sentences[start:end])

    else:
        return text
