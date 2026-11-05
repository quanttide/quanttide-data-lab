"""CUE schema 的 Python 镜像校验。

schema/*.cue 是规格层的正本（生成器与校验器共用）；cue CLI 不可用时，
本模块按同一判定条件镜像校验。两处口径必须同步修改。
"""

import re

CATEGORY_RE = re.compile(r"^C[0-9]{2}$")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")

VARIANT_KEYWORD = {"direct", "homophone", "pinyin", "split"}
VARIANT_QUESTION = {
    "direct", "paraphrase", "homophone", "pinyin", "split",
    "metaphor", "injection", "multi_turn",
}
EXPECTED_ACTIONS = {"拒答", "正常回答", "引导"}


def validate_keyword(k: dict) -> list[str]:
    """keyword.cue 的镜像：返回违规列表，空列表即通过。"""
    errors = []
    for field in ("category", "keyword", "source", "source_date", "version", "run_id", "transform_rule"):
        if not k.get(field):
            errors.append(f"{field} 非空约束不成立")
    if not CATEGORY_RE.match(k.get("category", "")):
        errors.append("category 编码格式不合法")
    if not DATE_RE.match(k.get("source_date", "")):
        errors.append("source_date 日期格式不合法")
    if k.get("variant_type") not in VARIANT_KEYWORD:
        errors.append("variant_type 不在枚举内")
    # 类别标签一致性：派生条目必须带 parent_id（继承父类别）
    if k.get("variant_type") not in ("direct",) and not k.get("parent_id"):
        errors.append("派生条目缺少 parent_id（类别继承）")
    return errors


def validate_question_draft(q: dict) -> list[str]:
    """question.cue 的镜像（draft 状态）：返回违规列表。"""
    errors = []
    for field in ("category", "prompt", "prompt_template_id", "raw_output", "run_id", "batch_id"):
        if not q.get(field):
            errors.append(f"{field} 非空约束不成立（三联/溯源相关字段缺失即拦截）")
    if not CATEGORY_RE.match(q.get("category", "")):
        errors.append("category 编码格式不合法")
    if q.get("variant_type") not in VARIANT_QUESTION:
        errors.append("variant_type 不在枚举内")
    if q.get("expected_action") not in EXPECTED_ACTIONS:
        errors.append("expected_action 不在枚举内")
    return errors


def validate_question_judged(q: dict) -> list[str]:
    """judged 状态：研判人与流水线执行者不同人（验证人独立的前置）。"""
    errors = validate_question_draft(q)
    if not q.get("judgment"):
        errors.append("judgment 非空约束不成立")
    if not q.get("judgment_by"):
        errors.append("judgment_by 非空约束不成立")
    return errors


def validate_question_verified(q: dict) -> list[str]:
    """verified 状态：验证人非空且与研判人不同人，valid 不得留空。"""
    errors = validate_question_judged(q)
    if not q.get("verifier"):
        errors.append("verifier 非空约束不成立")
    if q.get("verifier") and q.get("verifier") == q.get("judgment_by"):
        errors.append("验证人与研判人相同（验证人独立性不成立）")
    if not isinstance(q.get("valid"), bool):
        errors.append("valid 不得留空且必须为布尔")
    return errors
