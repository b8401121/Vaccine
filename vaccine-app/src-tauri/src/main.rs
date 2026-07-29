#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct DoseItem {
    dose_info: String,    // 例如 "第 1 劑", "第 2 劑", "追加劑 (第 5 劑)"
    timing_info: String,  // 例如 "出生 24 小時內", "滿 2 個月", "滿 5 歲至入學前"
    status: String,       // "Past", "Current", "Next"
    description: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct VaccineGroup {
    name: String,
    category: String,     // "Routine" 或 "HighRisk"
    audience: String,     // "Children" 或 "Adults"
    doses: Vec<DoseItem>,
}

#[derive(Serialize, Deserialize, Clone)]
struct VaccineResponse {
    age_display: String,
    groups: Vec<VaccineGroup>,
}

struct DoseSpec {
    min_month: i32,
    max_month: i32,
    dose_info: &'static str,
    timing_info: &'static str,
    description: &'static str,
}

struct GroupSpec {
    name: &'static str,
    category: &'static str,
    audience: &'static str,
    doses: Vec<DoseSpec>,
}

#[tauri::command]
fn get_eligible_vaccines(
    year: i32,
    month: u32,
    day: u32,
    is_roc: bool,
) -> Result<VaccineResponse, String> {
    let actual_year = if is_roc { year + 1911 } else { year };
    
    let dob = NaiveDate::from_ymd_opt(actual_year, month, day)
        .ok_or("無效的日期")?;
        
    let now = Local::now().naive_local().date();
    if dob > now {
        return Err("出生日期不能在未來".into());
    }

    let mut age_years = now.year() - dob.year();
    let mut age_months = now.month() as i32 - dob.month() as i32;
    let age_days = now.day() as i32 - dob.day() as i32;

    if age_days < 0 {
        age_months -= 1;
    }
    if age_months < 0 {
        age_years -= 1;
        age_months += 12;
    }

    let total_months = (age_years * 12) + age_months;

    let age_display = if age_years > 0 {
        if age_months > 0 {
            format!("{} 歲 {} 個月", age_years, age_months)
        } else {
            format!("{} 歲", age_years)
        }
    } else {
        format!("{} 個月", age_months.max(0))
    };

    let mut groups_out = Vec::new();

    // -- 兒童疫苗群組定義 (按年齡區間動態標註狀態) --
    if total_months <= 120 {
        let child_specs = vec![
            GroupSpec {
                name: "B 型肝炎疫苗 (Hep B)",
                category: "Routine",
                audience: "Children",
                doses: vec![
                    DoseSpec { min_month: 0, max_month: 1, dose_info: "第 1 劑", timing_info: "出生 24 小時內", description: "出生後儘速接種" },
                    DoseSpec { min_month: 1, max_month: 6, dose_info: "第 2 劑", timing_info: "出生滿 1 個月", description: "滿 1 個月施打" },
                    DoseSpec { min_month: 6, max_month: 120, dose_info: "第 3 劑", timing_info: "出生滿 6 個月", description: "滿 6 個月施打" },
                ],
            },
            GroupSpec {
                name: "五合一 / 百日咳混合疫苗 (DTaP-Hib-IPV / DTaP-IPV)",
                category: "Routine",
                audience: "Children",
                doses: vec![
                    DoseSpec { min_month: 2, max_month: 4, dose_info: "第 1 劑", timing_info: "出生滿 2 個月", description: "基礎劑第 1 劑" },
                    DoseSpec { min_month: 4, max_month: 6, dose_info: "第 2 劑", timing_info: "出生滿 4 個月", description: "基礎劑第 2 劑" },
                    DoseSpec { min_month: 6, max_month: 18, dose_info: "第 3 劑", timing_info: "出生滿 6 個月", description: "基礎劑第 3 劑" },
                    DoseSpec { min_month: 18, max_month: 60, dose_info: "第 4 劑 (追加劑)", timing_info: "出生滿 18 個月", description: "滿 18 個月追加" },
                    DoseSpec { min_month: 60, max_month: 85, dose_info: "第 5 劑 (追加劑)", timing_info: "滿 5 歲至國小入學前", description: "國小入學前完成" },
                ],
            },
            GroupSpec {
                name: "13 價結合型肺炎鏈球菌疫苗 (PCV13)",
                category: "Routine",
                audience: "Children",
                doses: vec![
                    DoseSpec { min_month: 2, max_month: 4, dose_info: "第 1 劑", timing_info: "出生滿 2 個月", description: "基礎劑第 1 劑" },
                    DoseSpec { min_month: 4, max_month: 12, dose_info: "第 2 劑", timing_info: "出生滿 4 個月", description: "基礎劑第 2 劑" },
                    DoseSpec { min_month: 12, max_month: 120, dose_info: "第 3 劑 (追加劑)", timing_info: "出生滿 12-15 個月", description: "滿 12-15 個月追加" },
                ],
            },
            GroupSpec {
                name: "卡介苗 (BCG)",
                category: "Routine",
                audience: "Children",
                doses: vec![
                    DoseSpec { min_month: 5, max_month: 9, dose_info: "單劑", timing_info: "出生滿 5-8 個月", description: "建議滿 5-8 個月接種" },
                ],
            },
            GroupSpec {
                name: "水痘疫苗 (Varicella)",
                category: "Routine",
                audience: "Children",
                doses: vec![
                    DoseSpec { min_month: 12, max_month: 120, dose_info: "第 1 劑", timing_info: "出生滿 12 個月", description: "滿 12 個月施打" },
                ],
            },
            GroupSpec {
                name: "麻疹腮腺炎德國麻疹混合疫苗 (MMR)",
                category: "Routine",
                audience: "Children",
                doses: vec![
                    DoseSpec { min_month: 12, max_month: 60, dose_info: "第 1 劑", timing_info: "出生滿 12 個月", description: "滿 12 個月施打" },
                    DoseSpec { min_month: 60, max_month: 85, dose_info: "第 2 劑", timing_info: "滿 5 歲至國小入學前", description: "入學前施打第 2 劑" },
                ],
            },
            GroupSpec {
                name: "日本腦炎疫苗 (JE)",
                category: "Routine",
                audience: "Children",
                doses: vec![
                    DoseSpec { min_month: 15, max_month: 27, dose_info: "第 1 劑", timing_info: "出生滿 15 個月", description: "滿 15 個月施打" },
                    DoseSpec { min_month: 27, max_month: 120, dose_info: "第 2 劑", timing_info: "出生滿 27 個月", description: "與第 1 劑隔至少 12 個月" },
                ],
            },
            GroupSpec {
                name: "A 型肝炎疫苗 (Hep A)",
                category: "Routine",
                audience: "Children",
                doses: vec![
                    DoseSpec { min_month: 18, max_month: 27, dose_info: "第 1 劑", timing_info: "出生滿 18 個月", description: "滿 18 個月施打" },
                    DoseSpec { min_month: 27, max_month: 120, dose_info: "第 2 劑", timing_info: "出生滿 27 個月", description: "與第 1 劑隔至少 6 個月" },
                ],
            },
        ];

        for g in child_specs {
            let mut doses = Vec::new();
            for d in g.doses {
                let status = if total_months >= d.max_month {
                    "Past"
                } else if total_months >= d.min_month && total_months < d.max_month {
                    "Current"
                } else {
                    "Next"
                };

                doses.push(DoseItem {
                    dose_info: d.dose_info.to_string(),
                    timing_info: d.timing_info.to_string(),
                    status: status.to_string(),
                    description: d.description.to_string(),
                });
            }

            groups_out.push(VaccineGroup {
                name: g.name.to_string(),
                category: g.category.to_string(),
                audience: g.audience.to_string(),
                doses,
            });
        }
    }

    // -- 成人疫苗群組 --
    if age_years >= 18 {
        // 常規系列
        groups_out.push(VaccineGroup {
            name: "季節性流感疫苗".to_string(),
            category: "Routine".to_string(),
            audience: "Adults".to_string(),
            doses: vec![DoseItem {
                dose_info: "每年 1 劑".to_string(),
                timing_info: "秋冬流感季".to_string(),
                status: "Current".to_string(),
                description: "所有成人每年建議施打 1 劑".to_string(),
            }],
        });

        groups_out.push(VaccineGroup {
            name: "新冠疫苗 (COVID-19)".to_string(),
            category: "Routine".to_string(),
            audience: "Adults".to_string(),
            doses: vec![DoseItem {
                dose_info: "依政策 1-2 劑".to_string(),
                timing_info: "定期追加".to_string(),
                status: "Current".to_string(),
                description: "提升對抗主流病毒株之免疫保護力".to_string(),
            }],
        });

        groups_out.push(VaccineGroup {
            name: "減量破傷風、白喉、百日咳疫苗 (Tdap)".to_string(),
            category: "Routine".to_string(),
            audience: "Adults".to_string(),
            doses: vec![DoseItem {
                dose_info: "第 1 劑 / 每10年追加".to_string(),
                timing_info: "成人常規".to_string(),
                status: "Current".to_string(),
                description: "建議完成 1 劑 Tdap，之後每 10 年追加 1 劑".to_string(),
            }],
        });

        if age_years >= 19 && age_years <= 45 {
            groups_out.push(VaccineGroup {
                name: "人類乳突病毒疫苗 (HPV)".to_string(),
                category: if age_years <= 26 { "Routine".to_string() } else { "HighRisk".to_string() },
                audience: "Adults".to_string(),
                doses: vec![
                    DoseItem { dose_info: "第 1 劑".to_string(), timing_info: "0 個月".to_string(), status: "Current".to_string(), description: "第 1 劑接種".to_string() },
                    DoseItem { dose_info: "第 2 劑".to_string(), timing_info: "2 個月後".to_string(), status: "Next".to_string(), description: "與第 1 劑隔 2 個月".to_string() },
                    DoseItem { dose_info: "第 3 劑".to_string(), timing_info: "6 個月後".to_string(), status: "Next".to_string(), description: "與第 1 劑隔 6 個月".to_string() },
                ],
            });
        }

        if age_years >= 50 {
            groups_out.push(VaccineGroup {
                name: "非活性帶狀疱疹疫苗 (Shingles)".to_string(),
                category: "Routine".to_string(),
                audience: "Adults".to_string(),
                doses: vec![
                    DoseItem { dose_info: "第 1 劑".to_string(), timing_info: "50 歲以上".to_string(), status: "Current".to_string(), description: "第 1 劑接種".to_string() },
                    DoseItem { dose_info: "第 2 劑".to_string(), timing_info: "隔 2-6 個月".to_string(), status: "Next".to_string(), description: "與第 1 劑間隔 2-6 個月".to_string() },
                ],
            });
        }

        if age_years >= 65 || (age_years >= 55 && age_years <= 64) {
            groups_out.push(VaccineGroup {
                name: "肺炎鏈球菌疫苗 (PCV13/20 & PPV23)".to_string(),
                category: "Routine".to_string(),
                audience: "Adults".to_string(),
                doses: vec![
                    DoseItem {
                        dose_info: "公費 1-2 劑".to_string(),
                        timing_info: if age_years >= 65 { "65 歲以上長者" } else { "55-64 歲原住民" }.to_string(),
                        status: "Current".to_string(),
                        description: "建議施打 1 劑 PCV20 或 PCV13 銜接 PPV23".to_string(),
                    },
                ],
            });
        }

        if age_years >= 75 || age_years >= 60 {
            groups_out.push(VaccineGroup {
                name: "呼吸道細胞融合病毒 (RSV) 疫苗".to_string(),
                category: if age_years >= 75 { "Routine".to_string() } else { "HighRisk".to_string() },
                audience: "Adults".to_string(),
                doses: vec![DoseItem {
                    dose_info: "1 劑".to_string(),
                    timing_info: if age_years >= 75 { "75 歲以上" } else { "60-74 歲高風險" }.to_string(),
                    status: "Current".to_string(),
                    description: "預防 RSV 引發之下呼吸道疾病".to_string(),
                }],
            });
        }

        // 高風險
        groups_out.push(VaccineGroup {
            name: "麻疹腮腺炎德國麻疹疫苗 (MMR)".to_string(),
            category: "HighRisk".to_string(),
            audience: "Adults".to_string(),
            doses: vec![DoseItem {
                dose_info: "1-2 劑".to_string(),
                timing_info: "1966年後出生/無抗體者".to_string(),
                status: "Current".to_string(),
                description: "育齡婦女、醫療人員或出國高風險對象補打".to_string(),
            }],
        });
        groups_out.push(VaccineGroup {
            name: "B 型肝炎疫苗".to_string(),
            category: "HighRisk".to_string(),
            audience: "Adults".to_string(),
            doses: vec![DoseItem {
                dose_info: "共 3 劑".to_string(),
                timing_info: "按 0-1-6 月時程".to_string(),
                status: "Current".to_string(),
                description: "經檢驗為 B 肝抗體陰性者建議自費補打".to_string(),
            }],
        });
        groups_out.push(VaccineGroup {
            name: "A 型肝炎疫苗".to_string(),
            category: "HighRisk".to_string(),
            audience: "Adults".to_string(),
            doses: vec![DoseItem {
                dose_info: "共 2 劑".to_string(),
                timing_info: "間隔 6-12 個月".to_string(),
                status: "Current".to_string(),
                description: "慢性肝病或頻繁赴流行地區者".to_string(),
            }],
        });
        groups_out.push(VaccineGroup {
            name: "M 痘 (Mpox) 疫苗".to_string(),
            category: "HighRisk".to_string(),
            audience: "Adults".to_string(),
            doses: vec![DoseItem {
                dose_info: "共 2 劑".to_string(),
                timing_info: "間隔 4 週".to_string(),
                status: "Current".to_string(),
                description: "暴露前/後預防與風險行為者施打".to_string(),
            }],
        });
    }

    Ok(VaccineResponse {
        age_display,
        groups: groups_out,
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_eligible_vaccines])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
