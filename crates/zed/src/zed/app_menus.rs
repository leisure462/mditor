use gpui::{App, Menu, MenuItem, OsAction};

pub fn app_menus(_cx: &mut App) -> Vec<Menu> {
    use zed_actions::Quit;

    let view_items = vec![
        MenuItem::action(
            "放大",
            zed_actions::IncreaseBufferFontSize { persist: false },
        ),
        MenuItem::action(
            "缩小",
            zed_actions::DecreaseBufferFontSize { persist: false },
        ),
        MenuItem::action(
            "重置缩放",
            zed_actions::ResetBufferFontSize { persist: false },
        ),
        MenuItem::action("重置所有缩放", zed_actions::ResetAllZoom { persist: false }),
        MenuItem::separator(),
        MenuItem::action("切换左侧停靠区", workspace::ToggleLeftDock),
        MenuItem::action("切换右侧停靠区", workspace::ToggleRightDock),
        MenuItem::action("切换底部停靠区", workspace::ToggleBottomDock),
        MenuItem::action("切换全部停靠区", workspace::ToggleAllDocks),
        MenuItem::submenu(Menu {
            name: "编辑器布局".into(),
            items: vec![
                MenuItem::action("向上拆分", workspace::SplitUp::default()),
                MenuItem::action("向下拆分", workspace::SplitDown::default()),
                MenuItem::action("向左拆分", workspace::SplitLeft::default()),
                MenuItem::action("向右拆分", workspace::SplitRight::default()),
            ],
        }),
        MenuItem::separator(),
        MenuItem::action("项目面板", zed_actions::project_panel::ToggleFocus),
        MenuItem::action("大纲面板", outline_panel::ToggleFocus),
        MenuItem::separator(),
        MenuItem::action("预览 Markdown", markdown_preview::OpenPreview),
        MenuItem::separator(),
    ];

    vec![
        Menu {
            name: "Prism".into(),
            items: vec![
                MenuItem::action("关于 Prism", zed_actions::About),
                MenuItem::separator(),
                MenuItem::submenu(Menu {
                    name: "设置".into(),
                    items: vec![
                        MenuItem::action("打开设置", zed_actions::OpenSettings),
                        MenuItem::action("打开设置文件", super::OpenSettingsFile),
                        MenuItem::action("打开项目设置", zed_actions::OpenProjectSettings),
                        MenuItem::action("打开项目设置文件", super::OpenProjectSettingsFile),
                        MenuItem::action("打开默认设置", super::OpenDefaultSettings),
                        MenuItem::separator(),
                        MenuItem::action(
                            "选择主题...",
                            zed_actions::theme_selector::Toggle::default(),
                        ),
                        MenuItem::action(
                            "选择图标主题...",
                            zed_actions::icon_theme_selector::Toggle::default(),
                        ),
                    ],
                }),
                MenuItem::separator(),
                #[cfg(target_os = "macos")]
                MenuItem::os_submenu("服务", gpui::SystemMenuType::Services),
                MenuItem::separator(),
                #[cfg(target_os = "macos")]
                MenuItem::action("隐藏 Prism", super::Hide),
                #[cfg(target_os = "macos")]
                MenuItem::action("隐藏其他应用", super::HideOthers),
                #[cfg(target_os = "macos")]
                MenuItem::action("显示全部", super::ShowAll),
                MenuItem::separator(),
                MenuItem::action("退出 Prism", Quit),
            ],
        },
        Menu {
            name: "文件".into(),
            items: vec![
                MenuItem::action("新建", workspace::NewFile),
                MenuItem::action("新建窗口", workspace::NewWindow),
                MenuItem::separator(),
                #[cfg(not(target_os = "macos"))]
                MenuItem::action("打开文件...", workspace::OpenFiles),
                MenuItem::action(
                    if cfg!(not(target_os = "macos")) {
                        "打开文件夹..."
                    } else {
                        "打开…"
                    },
                    workspace::Open::default(),
                ),
                MenuItem::action(
                    "打开最近项目...",
                    zed_actions::OpenRecent {
                        create_new_window: false,
                    },
                ),
                MenuItem::separator(),
                MenuItem::action("将文件夹添加到项目…", workspace::AddFolderToProject),
                MenuItem::separator(),
                MenuItem::action("保存", workspace::Save { save_intent: None }),
                MenuItem::action("另存为…", workspace::SaveAs),
                MenuItem::action("全部保存", workspace::SaveAll { save_intent: None }),
                MenuItem::separator(),
                MenuItem::action(
                    "关闭编辑器",
                    workspace::CloseActiveItem {
                        save_intent: None,
                        close_pinned: true,
                    },
                ),
                MenuItem::action("关闭项目", workspace::CloseProject),
                MenuItem::action("关闭窗口", workspace::CloseWindow),
            ],
        },
        Menu {
            name: "编辑".into(),
            items: vec![
                MenuItem::os_action("撤销", editor::actions::Undo, OsAction::Undo),
                MenuItem::os_action("重做", editor::actions::Redo, OsAction::Redo),
                MenuItem::separator(),
                MenuItem::os_action("剪切", editor::actions::Cut, OsAction::Cut),
                MenuItem::os_action("复制", editor::actions::Copy, OsAction::Copy),
                MenuItem::action("复制并裁剪", editor::actions::CopyAndTrim),
                MenuItem::os_action("粘贴", editor::actions::Paste, OsAction::Paste),
                MenuItem::separator(),
                MenuItem::action("查找", search::buffer_search::Deploy::find()),
                MenuItem::action("在项目中查找", workspace::DeploySearch::find()),
            ],
        },
        Menu {
            name: "选择".into(),
            items: vec![
                MenuItem::os_action("全选", editor::actions::SelectAll, OsAction::SelectAll),
                MenuItem::action("扩大选区", editor::actions::SelectLargerSyntaxNode),
                MenuItem::action("缩小选区", editor::actions::SelectSmallerSyntaxNode),
                MenuItem::action("选择下一个同级节点", editor::actions::SelectNextSyntaxNode),
                MenuItem::action(
                    "选择上一个同级节点",
                    editor::actions::SelectPreviousSyntaxNode,
                ),
                MenuItem::separator(),
                MenuItem::action(
                    "在上方添加光标",
                    editor::actions::AddSelectionAbove {
                        skip_soft_wrap: true,
                    },
                ),
                MenuItem::action(
                    "在下方添加光标",
                    editor::actions::AddSelectionBelow {
                        skip_soft_wrap: true,
                    },
                ),
                MenuItem::action(
                    "选择下一个匹配项",
                    editor::actions::SelectNext {
                        replace_newest: false,
                    },
                ),
                MenuItem::action(
                    "选择上一个匹配项",
                    editor::actions::SelectPrevious {
                        replace_newest: false,
                    },
                ),
                MenuItem::action("选择所有匹配项", editor::actions::SelectAllMatches),
                MenuItem::separator(),
                MenuItem::action("上移当前行", editor::actions::MoveLineUp),
                MenuItem::action("下移当前行", editor::actions::MoveLineDown),
                MenuItem::action("复制选区", editor::actions::DuplicateLineDown),
            ],
        },
        Menu {
            name: "查看".into(),
            items: view_items,
        },
        Menu {
            name: "前往".into(),
            items: vec![
                MenuItem::action("后退", workspace::GoBack),
                MenuItem::action("前进", workspace::GoForward),
                MenuItem::separator(),
                MenuItem::action("命令面板...", zed_actions::command_palette::Toggle),
                MenuItem::separator(),
                MenuItem::action("转到文件...", workspace::ToggleFileFinder::default()),
                // MenuItem::action("Go to Symbol in Project", project_symbols::Toggle),
                MenuItem::action("转到编辑器内符号...", zed_actions::outline::ToggleOutline),
                MenuItem::action("转到行/列...", editor::actions::ToggleGoToLine),
            ],
        },
        Menu {
            name: "窗口".into(),
            items: vec![
                MenuItem::action("最小化", super::Minimize),
                MenuItem::action("缩放", super::Zoom),
                MenuItem::separator(),
            ],
        },
        Menu {
            name: "帮助".into(),
            items: vec![
                MenuItem::action("查看依赖许可证", zed_actions::OpenLicenses),
                MenuItem::separator(),
                MenuItem::action(
                    "文档",
                    super::OpenBrowser {
                        url: "https://zed.dev/docs".into(),
                    },
                ),
            ],
        },
    ]
}
