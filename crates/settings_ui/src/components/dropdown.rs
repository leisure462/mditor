use std::rc::Rc;

use gpui::{App, ElementId, IntoElement, RenderOnce};
use heck::ToTitleCase as _;
use ui::{
    ButtonSize, ContextMenu, DropdownMenu, DropdownStyle, FluentBuilder as _, IconPosition, px,
};

fn localized_variant_label(label: &str) -> &str {
    let normalized = label
        .chars()
        .filter(|char| !matches!(char, ' ' | '_' | '-' | '/'))
        .collect::<String>();

    match normalized.as_str() {
        "On" => "开启",
        "Off" => "关闭",
        "Enabled" => "启用",
        "Disabled" => "禁用",
        "Auto" => "自动",
        "Always" => "始终",
        "Never" => "从不",
        "Left" => "左侧",
        "Right" => "右侧",
        "Top" => "顶部",
        "Bottom" => "底部",
        "Center" => "居中",
        "Static" => "固定",
        "Dynamic" => "动态",
        "System" => "跟随系统",
        "Light" => "浅色",
        "Dark" => "深色",
        "Comfortable" => "舒适",
        "Standard" => "标准",
        "Custom" => "自定义",
        "AfterDelay" => "延迟后",
        "OnFocusChange" => "失去焦点时",
        "OnWindowChange" => "窗口变化时",
        "PrimaryScreen" => "主屏幕",
        "AllScreens" => "所有屏幕",
        "CurrentScreen" => "当前屏幕",
        "ActiveScreen" => "活动屏幕",
        "PlatformNative" => "系统原生",
        "Code" => "代码",
        "File" => "文件",
        "Bar" => "条形",
        "Block" => "方块",
        "Underline" => "下划线",
        "Hollow" => "空心",
        "All" => "全部",
        "None" => "无",
        "Current" => "当前项",
        "Directory" => "目录",
        "Project" => "项目",
        "Hidden" => "隐藏",
        "Visible" => "可见",
        "Vertical" => "垂直",
        "Horizontal" => "水平",
        "Wrapped" => "按折行",
        "Open" => "打开",
        "Closed" => "关闭",
        "Docked" => "停靠",
        "Fullscreen" => "全屏",
        "Screen" => "屏幕",
        "Workspace" => "工作区",
        "Buffer" => "缓冲区",
        "Replace" => "替换",
        "Visual" => "可视",
        "Normal" => "普通",
        "Insert" => "插入",
        "Local" => "本地",
        "Global" => "全局",
        "Manual" => "手动",
        "Click" => "点击",
        "DoubleClick" => "双击",
        "Selection" => "选区",
        "First" => "第一个",
        "Last" => "最后一个",
        "Previous" => "上一个",
        "Next" => "下一个",
        "true" => "是",
        "false" => "否",
        _ => label,
    }
}

#[derive(IntoElement)]
pub struct EnumVariantDropdown<T>
where
    T: strum::VariantArray + strum::VariantNames + Copy + PartialEq + Send + Sync + 'static,
{
    id: ElementId,
    current_value: T,
    variants: &'static [T],
    labels: &'static [&'static str],
    should_do_title_case: bool,
    tab_index: Option<isize>,
    on_change: Rc<dyn Fn(T, &mut ui::Window, &mut App) + 'static>,
}

impl<T> EnumVariantDropdown<T>
where
    T: strum::VariantArray + strum::VariantNames + Copy + PartialEq + Send + Sync + 'static,
{
    pub fn new(
        id: impl Into<ElementId>,
        current_value: T,
        variants: &'static [T],
        labels: &'static [&'static str],
        on_change: impl Fn(T, &mut ui::Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            current_value,
            variants,
            labels,
            should_do_title_case: true,
            tab_index: None,
            on_change: Rc::new(on_change),
        }
    }

    pub fn title_case(mut self, title_case: bool) -> Self {
        self.should_do_title_case = title_case;
        self
    }

    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = Some(tab_index);
        self
    }
}

impl<T> RenderOnce for EnumVariantDropdown<T>
where
    T: strum::VariantArray + strum::VariantNames + Copy + PartialEq + Send + Sync + 'static,
{
    fn render(self, window: &mut ui::Window, cx: &mut ui::App) -> impl gpui::IntoElement {
        let current_value_label = self.labels[self
            .variants
            .iter()
            .position(|v| *v == self.current_value)
            .unwrap()];

        let context_menu = window.use_keyed_state(current_value_label, cx, |window, cx| {
            ContextMenu::new(window, cx, move |mut menu, _, _| {
                for (&value, &label) in std::iter::zip(self.variants, self.labels) {
                    let on_change = self.on_change.clone();
                    let current_value = self.current_value;
                    let localized = localized_variant_label(label);
                    menu = menu.toggleable_entry(
                        if self.should_do_title_case && localized == label {
                            localized.to_title_case()
                        } else {
                            localized.to_string()
                        },
                        value == current_value,
                        IconPosition::End,
                        None,
                        move |window, cx| {
                            on_change(value, window, cx);
                        },
                    );
                }
                menu
            })
        });

        DropdownMenu::new(
            self.id,
            {
                let localized = localized_variant_label(current_value_label);
                if self.should_do_title_case && localized == current_value_label {
                    localized.to_title_case()
                } else {
                    localized.to_string()
                }
            },
            context_menu,
        )
        .when_some(self.tab_index, |elem, tab_index| elem.tab_index(tab_index))
        .trigger_size(ButtonSize::Medium)
        .style(DropdownStyle::Outlined)
        .offset(gpui::Point {
            x: px(0.0),
            y: px(2.0),
        })
        .into_any_element()
    }
}
