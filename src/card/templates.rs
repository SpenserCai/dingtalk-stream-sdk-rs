//! 交互式卡片模板，对齐 Python interactive_card.py

/// 多行文本卡片模板
fn multi_text_line_template() -> serde_json::Value {
    serde_json::json!({
        "config": {"autoLayout": true, "enableForward": true},
        "header": {
            "title": {"type": "text", "text": "钉钉卡片"},
            "logo": "@lALPDfJ6V_FPDmvNAfTNAfQ"
        },
        "contents": [
            {"type": "markdown", "text": "钉钉正在为各行各业提供专业解决方案，沉淀钉钉1900万企业组织核心业务场景，提供专属钉钉、教育、医疗、新零售等多行业多维度的解决方案。", "id": "markdown_1686281949314"},
            {"type": "divider", "id": "divider_1686281949314"}
        ]
    })
}

/// 多行文本+图片卡片模板
fn multi_text_image_template() -> serde_json::Value {
    serde_json::json!({
        "config": {"autoLayout": true, "enableForward": true},
        "header": {
            "title": {"type": "text", "text": "钉钉卡片"},
            "logo": "@lALPDfJ6V_FPDmvNAfTNAfQ"
        },
        "contents": [
            {"type": "markdown", "text": "钉钉正在为各行各业提供专业解决方案，沉淀钉钉1900万企业组织核心业务场景，提供专属钉钉、教育、医疗、新零售等多行业多维度的解决方案。", "id": "markdown_1686281949314"},
            {"type": "divider", "id": "divider_1686281949314"},
            {"type": "imageList", "images": ["@lADPDe7s2ySi18PNA6XNBXg", "@lADPDf0i1beuNF3NAxTNBXg", "@lADPDe7s2ySRnIvNA6fNBXg"], "id": "imageList_1686283179480"}
        ]
    })
}

/// 极简卡片组合：title-text-image-button
pub fn interactive_card_sample_1() -> serde_json::Value {
    serde_json::json!({
        "config": {"autoLayout": true, "enableForward": true},
        "header": {
            "title": {"type": "text", "text": "钉钉卡片"},
            "logo": "@lALPDfJ6V_FPDmvNAfTNAfQ"
        },
        "contents": [
            {"type": "markdown", "text": "钉钉，让进步发生！\n 更新时间：2023-06-06 12:00", "id": "text_1686025745169"},
            {"type": "image", "image": "@lADPDetfXH_Pn3HNAbrNBDg", "id": "image_1686025745169"},
            {"type": "action", "actions": [
                {"type": "button", "label": {"type": "text", "text": "打开链接", "id": "text_1686025745289"}, "actionType": "openLink", "url": {"all": "https://www.dingtalk.com"}, "status": "primary", "id": "button_1646816888247"},
                {"type": "button", "label": {"type": "text", "text": "回传请求", "id": "text_1686025745208"}, "actionType": "request", "status": "primary", "id": "button_1646816888257"}
            ], "id": "action_1686025745169"}
        ]
    })
}

/// 较丰富的组件卡片：title-text-image-section-button
pub fn interactive_card_sample_2() -> serde_json::Value {
    serde_json::json!({
        "config": {"autoLayout": true, "enableForward": true},
        "header": {
            "title": {"type": "text", "text": "钉钉卡片"},
            "logo": "@lALPDfJ6V_FPDmvNAfTNAfQ"
        },
        "contents": [
            {"type": "markdown", "text": "钉钉正在为各行各业提供专业解决方案，沉淀钉钉1900万企业组织核心业务场景，提供专属钉钉、教育、医疗、新零售等多行业多维度的解决方案。", "id": "text_1686025745169"},
            {"type": "image", "image": "@lADPDetfXH_Pn3HNAbrNBDg", "id": "image_1686025745169"},
            {"type": "divider", "id": "divider_1686025745169"},
            {"type": "section", "fields": {"list": [
                {"type": "text", "text": "钉钉发起\u{201c}C10圆桌派\u{201d}，旨在邀请各行各业的CIO、CTO等，面对面深入交流数字化建设心得，总结行业…", "id": "text_1686025745205"},
                {"type": "text", "text": "在后疫情时期，数字化跃升为时代命题之一，混合办公及云上创新逐渐普及，数字化已成为企业发展的必答…", "id": "text_1686025745174"}
            ]}, "extra": {"type": "button", "label": {"type": "text", "text": "查看详情", "id": "text_1686025745191"}, "actionType": "openLink", "url": {"all": "https://alidocs.dingtalk.com/i/p/nb9XJlvOKbAyDGyA/docs/nb9XJo9ogo27lmyA?spm=a217n7.14136887.0.0.499d573fCVWe7p"}, "status": "primary", "id": "button_1646816886531"}, "id": "section_1686025745169"},
            {"type": "action", "actions": [
                {"type": "button", "label": {"type": "text", "text": "打开链接", "id": "text_1686025745289"}, "actionType": "openLink", "url": {"all": "https://www.dingtalk.com"}, "status": "primary", "id": "button_1646816888247"},
                {"type": "button", "label": {"type": "text", "text": "回传请求", "id": "text_1686025745208"}, "actionType": "request", "status": "primary", "id": "button_1646816888257"},
                {"type": "button", "label": {"type": "text", "text": "次级按钮", "id": "text_1686025745206"}, "actionType": "openLink", "url": {"all": "https://www.dingtalk.com"}, "status": "normal", "id": "button_1646816888277"},
                {"type": "button", "label": {"type": "text", "text": "警示按钮", "id": "text_1686025745195"}, "actionType": "openLink", "url": {"all": "https://www.dingtalk.com"}, "status": "warning", "id": "button_1646816888287"}
            ], "id": "action_1686025745169"}
        ]
    })
}

/// 较丰富的组件卡片：title-image-markdown-button
pub fn interactive_card_sample_3() -> serde_json::Value {
    serde_json::json!({
        "config": {"autoLayout": true, "enableForward": true},
        "header": {
            "title": {"type": "text", "text": "钉钉小技巧"},
            "logo": "@lALPDefR3hjhflFAQA"
        },
        "contents": [
            {"type": "image", "image": "@lALPDsCJC34CVxzNAYTNArA", "id": "image_1686034081551"},
            {"type": "markdown", "text": "🎉 **四招教你玩转钉钉项目**", "id": "markdown_1686034081551"},
            {"type": "markdown", "text": "一、创建项目群，重要事项放项目", "id": "markdown_1686034081584"},
            {"type": "markdown", "text": "😭  群内信息太碎片？任务交办难跟踪？协作边界很模糊？\n👉  试试创建项目群，把重要事项放在项目内跟踪，可以事半功倍！", "id": "markdown_1686034081625"},
            {"type": "markdown", "text": "<font size=12 color=common_level3_base_color>更多精彩内容请查看详情…</font>", "id": "markdown_1686034081660"},
            {"type": "action", "actions": [
                {"type": "button", "label": {"type": "text", "text": "查看详情", "id": "text_1686034081551"}, "actionType": "openLink", "url": {"all": "https://www.dingtalk.com"}, "status": "normal", "id": "button_1647166782413"}
            ], "id": "action_1686034081551"}
        ]
    })
}

/// 多行文本卡片模板
pub fn interactive_card_multi_text_line() -> serde_json::Value {
    multi_text_line_template()
}

/// 多行文本+图片卡片模板
pub fn interactive_card_multi_text_image() -> serde_json::Value {
    multi_text_image_template()
}

/// 动态生成多行文本卡片数据
pub fn generate_multi_text_line_card_data(
    title: &str,
    logo: &str,
    texts: &[&str],
) -> serde_json::Value {
    let mut card_data = multi_text_line_template();

    if !title.is_empty() {
        card_data["header"]["title"]["text"] = serde_json::json!(title);
    }
    if !logo.is_empty() {
        card_data["header"]["logo"] = serde_json::json!(logo);
    }

    let mut contents = Vec::new();
    for text in texts {
        contents.push(serde_json::json!({
            "type": "markdown",
            "text": text,
            "id": format!("text_{}", uuid::Uuid::new_v4())
        }));
        contents.push(serde_json::json!({
            "type": "divider",
            "id": format!("divider_{}", uuid::Uuid::new_v4())
        }));
    }
    card_data["contents"] = serde_json::json!(contents);

    card_data
}

/// 动态生成多行文本+图片卡片数据
pub fn generate_multi_text_image_card_data(
    title: &str,
    logo: &str,
    texts: &[&str],
    images: &[&str],
) -> serde_json::Value {
    let mut card_data = multi_text_image_template();

    if !title.is_empty() {
        card_data["header"]["title"]["text"] = serde_json::json!(title);
    }
    if !logo.is_empty() {
        card_data["header"]["logo"] = serde_json::json!(logo);
    }

    let mut contents = Vec::new();
    for text in texts {
        contents.push(serde_json::json!({
            "type": "markdown",
            "text": text,
            "id": format!("text_{}", uuid::Uuid::new_v4())
        }));
        contents.push(serde_json::json!({
            "type": "divider",
            "id": format!("divider_{}", uuid::Uuid::new_v4())
        }));
    }

    let image_list: Vec<&str> = images.to_vec();
    contents.push(serde_json::json!({
        "type": "imageList",
        "images": image_list,
        "id": format!("imageList_{}", uuid::Uuid::new_v4())
    }));

    card_data["contents"] = serde_json::json!(contents);

    card_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_card_sample_1() {
        let card = interactive_card_sample_1();
        assert!(card["config"]["autoLayout"].as_bool().unwrap());
        assert_eq!(card["header"]["title"]["text"], "钉钉卡片");
    }

    #[test]
    fn test_generate_multi_text_line() {
        let card = generate_multi_text_line_card_data("Test Title", "@logo", &["Line 1", "Line 2"]);
        assert_eq!(card["header"]["title"]["text"], "Test Title");
        assert_eq!(card["header"]["logo"], "@logo");
        let contents = card["contents"].as_array().unwrap();
        // 2 texts * 2 (text + divider) = 4
        assert_eq!(contents.len(), 4);
        assert_eq!(contents[0]["type"], "markdown");
        assert_eq!(contents[0]["text"], "Line 1");
        assert_eq!(contents[1]["type"], "divider");
    }

    #[test]
    fn test_generate_multi_text_image() {
        let card =
            generate_multi_text_image_card_data("Title", "@logo", &["Text 1"], &["@img1", "@img2"]);
        let contents = card["contents"].as_array().unwrap();
        // 1 text * 2 + 1 imageList = 3
        assert_eq!(contents.len(), 3);
        assert_eq!(contents[2]["type"], "imageList");
        let images = contents[2]["images"].as_array().unwrap();
        assert_eq!(images.len(), 2);
    }
}
