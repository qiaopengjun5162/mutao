import pytest
from scalpel import extract_tags_local, extract_tags_llm


class TestExtractTagsLocal:
    def test_keyboard(self):
        result = extract_tags_local("机械键盘 Cherry 轴")
        assert "机械键盘" in result["tags"]
        assert result["method"] == "local_rule"

    def test_book(self):
        result = extract_tags_local("三体小说 全套")
        assert "书籍" in result["tags"]

    def test_headphone(self):
        result = extract_tags_local("索尼降噪耳机 WH-1000XM4")
        assert "耳机" in result["tags"]

    def test_camera_high_value(self):
        result = extract_tags_local("索尼 A7M3 相机")
        assert "相机" in result["tags"]
        assert result["value_tier"] == 4

    def test_keyboard_mid_value(self):
        result = extract_tags_local("机械键盘")
        assert result["value_tier"] == 3

    def test_unknown_item(self):
        result = extract_tags_local("一个普通水杯")
        assert "其他" in result["tags"]
        assert result["value_tier"] == 1

    def test_multiple_tags(self):
        result = extract_tags_local("书桌和椅子")
        assert "家具" in result["tags"]

    def test_returns_method(self):
        result = extract_tags_local("测试")
        assert result["method"] == "local_rule"


class TestExtractTagsLLM:
    def test_fallback_to_local(self, monkeypatch):
        """LLM 调用失败时降级到本地规则"""
        monkeypatch.setattr("scalpel.LLM_API_KEY", "")
        result = extract_tags_llm("机械键盘")
        assert result["method"] == "local_rule"

    def test_returns_dict_structure(self):
        """确保返回结构正确"""
        result = extract_tags_local("测试物品")
        assert "tags" in result
        assert "value_tier" in result
        assert "method" in result
        assert isinstance(result["tags"], list)
        assert isinstance(result["value_tier"], int)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
