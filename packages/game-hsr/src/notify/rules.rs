use crate::api::widget::WidgetData;

/// Check all notification rules and fire system notifications
pub fn check_rules(
    data: &WidgetData,
    old: Option<&WidgetData>,
    app: &tauri::AppHandle,
) {
    // 1. Stamina nearly full (>80%)
    if data.max_stamina > 0 {
        let pct = data.current_stamina as f64 / data.max_stamina as f64;
        if pct >= 0.95 {
            notify(app, "\u{4f53}\u{529b}\u{5feb}\u{6ee1}\u{4e86}", &format!("\u{5f53}\u{524d} {}/{}", data.current_stamina, data.max_stamina));
        } else if pct >= 0.80 {
            if let Some(old) = old {
                let old_pct = old.current_stamina as f64 / old.max_stamina as f64;
                if old_pct < 0.80 {
                    notify(app, "\u{4f53}\u{529b}\u{8d85}\u{8fc7}80%", &format!("\u{5f53}\u{524d} {}/{}", data.current_stamina, data.max_stamina));
                }
            }
        }
    }

    // 2. Expeditions all completed
    if data.total_expedition_num > 0 && data.accepted_expedition_num == 0 {
        if let Some(old) = old {
            if old.accepted_expedition_num > 0 {
                notify(app, "\u{6d3e}\u{9063}\u{5168}\u{90e8}\u{5b8c}\u{6210}", "\u{6240}\u{6709}\u{59d4}\u{6258}\u{5df2}\u{8fd4}\u{56de}");
            }
        }
    }

    // 3. Reserve stamina full
    if data.is_reserve_stamina_full {
        notify(app, "\u{5907}\u{7528}\u{4f53}\u{529b}\u{5df2}\u{6ee1}", "\u{8bf7}\u{53ca}\u{65f6}\u{4f7f}\u{7528}");
    }

    // 4. Daily not signed in
    if !data.has_signed {
        if let Some(old) = old {
            if old.has_signed {
                notify(app, "\u{4eca}\u{65e5}\u{672a}\u{7b7e}\u{5230}", "\u{661f}\u{7a79}\u{94c1}\u{9053}\u{4eca}\u{65e5}\u{8fd8}\u{672a}\u{7b7e}\u{5230}");
            }
        }
    }

    // 5. Simulated universe not done this week
    if data.max_rogue_score > 0 && data.current_rogue_score == 0 {
        if let Some(old) = old {
            if old.current_rogue_score > 0 {
                notify(app, "\u{6a21}\u{62df}\u{5b87}\u{5b99}\u{672a}\u{6253}", "\u{672c}\u{5468}\u{6a21}\u{62df}\u{5b87}\u{5b99}\u{79ef}\u{5206}\u{8fd8}\u{672a}\u{83b7}\u{53d6}");
            }
        }
    }
}

fn notify(app: &tauri::AppHandle, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;
    if let Err(e) = app.notification().builder()
        .title(title)
        .body(body)
        .show()
    {
        log::error!("Notification error: {}", e);
    }
}
