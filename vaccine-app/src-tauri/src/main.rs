#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct VaccineItem {
    name: String,
    dose_info: String,
    timing_info: String,
    category: String, // "Routine" (公費/常規) 或 "SelfPaid" (自費/建議)
    description: String,
    audience: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct TimelineMilestone {
    title: String,
    age_months: i32,
    status: String, // "Past", "Current", "Next"
    vaccines: Vec<VaccineItem>,
}

#[derive(Serialize, Deserialize, Clone)]
struct VaccineResponse {
    age_display: String,
    child_age_detail: String,
    gender_display: String,
    milestones: Vec<TimelineMilestone>,
}

#[derive(Serialize, Deserialize, Clone)]
struct VaccineDetailDoc {
    id: String,
    name: String,
    aliases: String,
    category: String, // "Routine" | "SelfPaid" | "Both"
    target_audience: String, // "兒童/青少年" | "成人/長者" | "全齡通用"
    prevent_disease: String,
    full_description: String,
    schedule: Vec<String>,
    notes: String,
}

struct MilestoneSpec {
    title: &'static str,
    min_month: i32,
    max_month: i32,
    vaccines: Vec<VaccineItem>,
}

#[tauri::command]
fn get_eligible_vaccines(
    year: i32,
    month: u32,
    day: u32,
    is_roc: bool,
    gender: String,
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

    let child_age_detail = if total_months <= 216 {
        if age_years > 0 {
            format!("{} 歲 {} 個月 (相當於 {} 個月大)", age_years, age_months, total_months)
        } else {
            format!("{} 個月大", age_months.max(0))
        }
    } else {
        format!("{} 歲", age_years)
    };

    let is_female = gender == "female";
    let gender_display = if is_female { "女性 ♀" } else { "男性 ♂" }.to_string();

    let mut milestones_out = Vec::new();

    let mut child_specs = vec![
        MilestoneSpec {
            title: "出生 24 小時內",
            min_month: 0,
            max_month: 1,
            vaccines: vec![VaccineItem {
                name: "B 型肝炎疫苗".into(),
                dose_info: "第 1 劑".into(),
                timing_info: "出生 24 小時內".into(),
                category: "Routine".into(),
                description: "新生兒出生後儘速施打公費第 1 劑".into(),
                audience: "Children".into(),
            }],
        },
        MilestoneSpec {
            title: "出生滿 1 個月",
            min_month: 1,
            max_month: 2,
            vaccines: vec![VaccineItem {
                name: "B 型肝炎疫苗".into(),
                dose_info: "第 2 劑".into(),
                timing_info: "滿 1 個月".into(),
                category: "Routine".into(),
                description: "公費基礎劑第 2 劑".into(),
                audience: "Children".into(),
            }],
        },
        MilestoneSpec {
            title: "出生滿 2 個月",
            min_month: 2,
            max_month: 4,
            vaccines: vec![
                VaccineItem {
                    name: "五合一疫苗 (DTaP-Hib-IPV)".into(),
                    dose_info: "第 1 劑".into(),
                    timing_info: "滿 2 個月".into(),
                    category: "Routine".into(),
                    description: "公費預防白喉、破傷風、百日咳、B型嗜血桿菌、小兒麻痺".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "13 價結合型肺炎鏈球菌疫苗 (PCV13)".into(),
                    dose_info: "第 1 劑".into(),
                    timing_info: "滿 2 個月".into(),
                    category: "Routine".into(),
                    description: "公費基礎劑第 1 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "輪狀病毒疫苗 (Rotavirus)".into(),
                    dose_info: "自費口服第 1 劑".into(),
                    timing_info: "滿 2 個月".into(),
                    category: "SelfPaid".into(),
                    description: "自費口服疫苗 (2劑型或3劑型)，預防輪狀病毒嚴重腹瀉".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "腸病毒 A71 型疫苗 (EV71)".into(),
                    dose_info: "自費第 1 劑".into(),
                    timing_info: "滿 2 個月至 6 歲".into(),
                    category: "SelfPaid".into(),
                    description: "自費接種，預防腸病毒 A71 型併發重症".into(),
                    audience: "Children".into(),
                },
            ],
        },
        MilestoneSpec {
            title: "出生滿 4 個月",
            min_month: 4,
            max_month: 5,
            vaccines: vec![
                VaccineItem {
                    name: "五合一疫苗 (DTaP-Hib-IPV)".into(),
                    dose_info: "第 2 劑".into(),
                    timing_info: "滿 4 個月".into(),
                    category: "Routine".into(),
                    description: "公費基礎劑第 2 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "13 價結合型肺炎鏈球菌疫苗 (PCV13)".into(),
                    dose_info: "第 2 劑".into(),
                    timing_info: "滿 4 個月".into(),
                    category: "Routine".into(),
                    description: "公費基礎劑第 2 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "輪狀病毒疫苗 (Rotavirus)".into(),
                    dose_info: "自費口服第 2 劑".into(),
                    timing_info: "滿 4 個月".into(),
                    category: "SelfPaid".into(),
                    description: "自費口服第 2 劑 (2劑型此劑完成，3劑型需於6個月再服)".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "腸病毒 A71 型疫苗 (EV71)".into(),
                    dose_info: "自費第 2 劑".into(),
                    timing_info: "滿 4 個月 (與第1劑隔2月)".into(),
                    category: "SelfPaid".into(),
                    description: "自費基礎劑第 2 劑".into(),
                    audience: "Children".into(),
                },
            ],
        },
        MilestoneSpec {
            title: "出生滿 5-8 個月",
            min_month: 5,
            max_month: 6,
            vaccines: vec![VaccineItem {
                name: "卡介苗 (BCG)".into(),
                dose_info: "單劑".into(),
                timing_info: "滿 5-8 個月".into(),
                category: "Routine".into(),
                description: "公費建議於滿 5-8 個月施打".into(),
                audience: "Children".into(),
            }],
        },
        MilestoneSpec {
            title: "出生滿 6 個月",
            min_month: 6,
            max_month: 12,
            vaccines: vec![
                VaccineItem {
                    name: "五合一疫苗 (DTaP-Hib-IPV)".into(),
                    dose_info: "第 3 劑".into(),
                    timing_info: "滿 6 個月".into(),
                    category: "Routine".into(),
                    description: "公費基礎劑第 3 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "B 型肝炎疫苗".into(),
                    dose_info: "第 3 劑".into(),
                    timing_info: "滿 6 個月".into(),
                    category: "Routine".into(),
                    description: "公費基礎劑第 3 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "季節性流感疫苗".into(),
                    dose_info: "第 1-2 劑 (初次接種隔4週)".into(),
                    timing_info: "滿 6 個月以上".into(),
                    category: "Routine".into(),
                    description: "滿6個月即可接種流感疫苗，8歲以下初次接種需打2劑".into(),
                    audience: "Children".into(),
                },
            ],
        },
        MilestoneSpec {
            title: "出生滿 12 個月",
            min_month: 12,
            max_month: 15,
            vaccines: vec![
                VaccineItem {
                    name: "水痘疫苗 (Varicella)".into(),
                    dose_info: "第 1 劑".into(),
                    timing_info: "滿 12 個月".into(),
                    category: "Routine".into(),
                    description: "公費滿 12 個月施打第 1 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "麻疹腮腺炎德國麻疹混合疫苗 (MMR)".into(),
                    dose_info: "第 1 劑".into(),
                    timing_info: "滿 12 個月".into(),
                    category: "Routine".into(),
                    description: "公費滿 12 個月施打第 1 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "13 價結合型肺炎鏈球菌疫苗 (PCV13)".into(),
                    dose_info: "第 3 劑 (追加劑)".into(),
                    timing_info: "滿 12-15 個月".into(),
                    category: "Routine".into(),
                    description: "公費滿 12-15 個月施打追加劑".into(),
                    audience: "Children".into(),
                },
            ],
        },
        MilestoneSpec {
            title: "出生滿 15 個月",
            min_month: 15,
            max_month: 18,
            vaccines: vec![VaccineItem {
                name: "日本腦炎疫苗 (JE)".into(),
                dose_info: "第 1 劑".into(),
                timing_info: "滿 15 個月".into(),
                category: "Routine".into(),
                description: "公費滿 15 個月施打第 1 劑".into(),
                audience: "Children".into(),
            }],
        },
        MilestoneSpec {
            title: "出生滿 18 個月",
            min_month: 18,
            max_month: 27,
            vaccines: vec![
                VaccineItem {
                    name: "五合一疫苗 (DTaP-Hib-IPV)".into(),
                    dose_info: "第 4 劑 (追加劑)".into(),
                    timing_info: "滿 18 個月".into(),
                    category: "Routine".into(),
                    description: "公費滿 18 個月追加第 4 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "A 型肝炎疫苗 (Hep A)".into(),
                    dose_info: "第 1 劑".into(),
                    timing_info: "滿 18 個月".into(),
                    category: "Routine".into(),
                    description: "公費滿 18 個月施打第 1 劑".into(),
                    audience: "Children".into(),
                },
            ],
        },
        MilestoneSpec {
            title: "出生滿 27 個月",
            min_month: 27,
            max_month: 60,
            vaccines: vec![
                VaccineItem {
                    name: "日本腦炎疫苗 (JE)".into(),
                    dose_info: "第 2 劑".into(),
                    timing_info: "滿 27 個月".into(),
                    category: "Routine".into(),
                    description: "公費與第 1 劑隔至少 12 個月".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "A 型肝炎疫苗 (Hep A)".into(),
                    dose_info: "第 2 劑".into(),
                    timing_info: "滿 27 個月".into(),
                    category: "Routine".into(),
                    description: "公費與第 1 劑隔至少 6 個月".into(),
                    audience: "Children".into(),
                },
            ],
        },
        MilestoneSpec {
            title: "滿 4-6 歲幼兒追加自費",
            min_month: 48,
            max_month: 60,
            vaccines: vec![VaccineItem {
                name: "水痘疫苗 (Varicella)".into(),
                dose_info: "自費第 2 劑 (追加劑)".into(),
                timing_info: "滿 4-6 歲".into(),
                category: "SelfPaid".into(),
                description: "小兒感染症醫學會建議於 4-6 歲自費追加第 2 劑水痘疫苗，保護力更佳".into(),
                audience: "Children".into(),
            }],
        },
        MilestoneSpec {
            title: "滿 5 歲至國小入學前",
            min_month: 60,
            max_month: 108,
            vaccines: vec![
                VaccineItem {
                    name: "麻疹腮腺炎德國麻疹混合疫苗 (MMR)".into(),
                    dose_info: "第 2 劑".into(),
                    timing_info: "滿 5 歲至國小入學前".into(),
                    category: "Routine".into(),
                    description: "公費入學前完成施打第 2 劑".into(),
                    audience: "Children".into(),
                },
                VaccineItem {
                    name: "白喉破傷風百日咳及小兒麻痺疫苗 (DTaP-IPV)".into(),
                    dose_info: "追加劑 (第 5 劑)".into(),
                    timing_info: "滿 5 歲至國小入學前".into(),
                    category: "Routine".into(),
                    description: "公費入學前追加 1 劑".into(),
                    audience: "Children".into(),
                },
            ],
        },
    ];

    let hpv_adolescent_routine = if is_female {
        vec![
            VaccineItem {
                name: "人類乳突病毒疫苗 (HPV)".into(),
                dose_info: "公費 2 劑 (按 0, 6 個月)".into(),
                timing_info: "國中女生 (約12-15歲)".into(),
                category: "Routine".into(),
                description: "衛福部國健署提供國中女生公費施打 2 劑 9 價 HPV 疫苗，有效預防子宮頸癌及 HPV 相關癌症".into(),
                audience: "Children".into(),
            },
            VaccineItem {
                name: "人類乳突病毒疫苗 (HPV)".into(),
                dose_info: "自費 2 劑 (按 0, 6 個月)".into(),
                timing_info: "9-14 歲青少年".into(),
                category: "SelfPaid".into(),
                description: "非公費補助資格之 9-14 歲女生，可選擇自費接種 2 劑 9 價 HPV 疫苗".into(),
                audience: "Children".into(),
            },
        ]
    } else {
        vec![VaccineItem {
            name: "人類乳突病毒疫苗 (HPV)".into(),
            dose_info: "自費 2 劑 (按 0, 6 個月)".into(),
            timing_info: "9-14 歲男性青少年".into(),
            category: "SelfPaid".into(),
            description: "9-14 歲男性建議自費接種 2 劑 4價或9價 HPV 疫苗，預防尖形濕疣(菜花)及陰莖癌/肛門癌等".into(),
            audience: "Children".into(),
        }]
    };

    child_specs.push(MilestoneSpec {
        title: "滿 9 歲至 14 歲 (青少年/國中階段)",
        min_month: 108,
        max_month: 180,
        vaccines: hpv_adolescent_routine,
    });

    let hpv_highschool = vec![VaccineItem {
        name: "人類乳突病毒疫苗 (HPV)".into(),
        dose_info: "自費 3 劑 (按 0, 2, 6 個月)".into(),
        timing_info: "15 歲以上青少年".into(),
        category: "SelfPaid".into(),
        description: if is_female {
            "15歲以上首次接種需施打 3 劑 (按 0-2-6 個月時程)，防範 HPV 病毒感染及子宮頸癌"
        } else {
            "15歲以上男性首次接種需施打 3 劑 (按 0-2-6 個月時程)，防範 HPV 病毒感染及口咽癌/菜花"
        }.into(),
        audience: "Children".into(),
    }];

    child_specs.push(MilestoneSpec {
        title: "滿 15 歲至 18 歲 (高中/青年階段)",
        min_month: 180,
        max_month: 216,
        vaccines: hpv_highschool,
    });

    if total_months <= 216 {
        for spec in child_specs {
            let status = if total_months >= spec.max_month {
                "Past"
            } else if total_months >= spec.min_month && total_months < spec.max_month {
                "Current"
            } else {
                "Next"
            };

            milestones_out.push(TimelineMilestone {
                title: spec.title.to_string(),
                age_months: spec.min_month,
                status: status.to_string(),
                vaccines: spec.vaccines,
            });
        }
    }

    if age_years >= 18 {
        let mut adult_routine = vec![
            VaccineItem {
                name: "季節性流感疫苗".into(),
                dose_info: "每年 1 劑".into(),
                timing_info: "秋冬流感季".into(),
                category: "Routine".into(),
                description: "所有成人每年建議施打 1 劑 (公費或自費選用優質疫苗)".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "新冠疫苗 (COVID-19)".into(),
                dose_info: "依政策 1-2 劑".into(),
                timing_info: "定期追加".into(),
                category: "Routine".into(),
                description: "提升對抗主流病毒株之免疫保護力".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "減量破傷風、白喉、百日咳疫苗 (Tdap)".into(),
                dose_info: "自費第 1 劑 / 每10年追加".into(),
                timing_info: "成人常規".into(),
                category: "SelfPaid".into(),
                description: if is_female {
                    "成人建議每10年自費追加1劑。女性每次懷孕(27-36週)均建議施打1劑以傳遞抗體給嬰兒。"
                } else {
                    "成人建議每10年自費追加1劑，預防破傷風、白喉與百日咳。"
                }.into(),
                audience: "Adults".into(),
            },
        ];

        if age_years >= 19 && age_years <= 45 {
            let hpv_desc = if is_female {
                "自費HPV疫苗：2價僅適用女性，4價與9價適用男女性。可預防子宮頸癌及相關病變。(按 0-2-6 個月時程施打 3 劑)"
            } else {
                "自費HPV疫苗：4價與9價適用於男性。可預防尖形濕疣(菜花)及肛門癌等病變。(按 0-2-6 個月時程施打 3 劑)"
            };

            adult_routine.push(VaccineItem {
                name: "人類乳突病毒疫苗 (HPV)".into(),
                dose_info: "自費共 3 劑 (0-2-6 個月)".into(),
                timing_info: if age_years <= 26 { "19-26 歲建議" } else { "27-45 歲自費" }.into(),
                category: "SelfPaid".into(),
                description: hpv_desc.into(),
                audience: "Adults".into(),
            });
        }

        if age_years >= 50 || age_years >= 18 {
            adult_routine.push(VaccineItem {
                name: "非活性帶狀疱疹疫苗 (Shingrix)".into(),
                dose_info: "自費共 2 劑 (隔 2-6 月)".into(),
                timing_info: "50 歲以上或 18 歲以上高風險".into(),
                category: "SelfPaid".into(),
                description: "預防帶狀疱疹(皮蛇)及疱疹後神經痛，防護率達90%以上".into(),
                audience: "Adults".into(),
            });
        }

        if age_years >= 65 || (age_years >= 55 && age_years <= 64) {
            adult_routine.push(VaccineItem {
                name: "肺炎鏈球菌疫苗 (PCV13/PCV20/PPV23)".into(),
                dose_info: "公費/自費 1-2 劑".into(),
                timing_info: if age_years >= 65 { "65 歲以上" } else { "55-64 歲原住民" }.into(),
                category: "Routine".into(),
                description: "公費提供 1 劑 PCV13/PCV20 銜接 PPV23。未達公費年齡者可自費接種".into(),
                audience: "Adults".into(),
            });
        }

        if age_years >= 60 {
            let rsv_desc = if is_female {
                "60歲以上長者建議自費1劑。懷孕婦女(28-36週)接種可傳遞被動免疫給嬰兒。"
            } else {
                "60歲以上長者建議自費1劑，預防RSV引發之下呼吸道疾病與重症。"
            };

            adult_routine.push(VaccineItem {
                name: "呼吸道細胞融合病毒 (RSV) 疫苗".into(),
                dose_info: "自費 1 劑".into(),
                timing_info: "60 歲以上長者".into(),
                category: "SelfPaid".into(),
                description: rsv_desc.into(),
                audience: "Adults".into(),
            });
        }

        milestones_out.push(TimelineMilestone {
            title: format!("成人常規與自費建議疫苗 ({}, {})", age_display, gender_display),
            age_months: total_months,
            status: "Current".to_string(),
            vaccines: adult_routine,
        });

        let travel_specs = vec![
            VaccineItem {
                name: "A 型肝炎疫苗".into(),
                dose_info: "自費共 2 劑 (隔 6-12 個月)".into(),
                timing_info: "高風險/赴流行區".into(),
                category: "SelfPaid".into(),
                description: "慢性肝病或頻繁赴東南亞/大陸等流行地區者建議自費接種".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "流行性腦脊髓膜炎疫苗 (MenACWY / MenB)".into(),
                dose_info: "自費 1-2 劑".into(),
                timing_info: "赴中東/留學/高風險區".into(),
                category: "SelfPaid".into(),
                description: "沙烏地朝聖或赴歐美留學特定學校強制要求接種之自費疫苗".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "黃熱病疫苗 (Yellow Fever)".into(),
                dose_info: "自費 1 劑 (終生有效)".into(),
                timing_info: "赴非洲/南美洲前10天".into(),
                category: "SelfPaid".into(),
                description: "旅遊醫學門診專用自費疫苗，入境特定國家所需之國際預防接種證明(黃皮書)".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "傷寒疫苗 (Typhoid Vaccine)".into(),
                dose_info: "自費 1 劑 (效期 3 年)".into(),
                timing_info: "赴流行區前 2 週".into(),
                category: "SelfPaid".into(),
                description: "旅遊醫學門診自費疫苗，前往高風險衛生不良地區旅遊前接種".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "狂犬病疫苗 (Rabies Vaccine)".into(),
                dose_info: "自費 3 劑 (按 0, 7, 21-28 天)".into(),
                timing_info: "暴露前預防/高風險工作".into(),
                category: "SelfPaid".into(),
                description: "獸醫、野生動物研究人員或前往狂犬病高風險國家野生動物接觸者".into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "麻疹腮腺炎德國麻疹疫苗 (MMR)".into(),
                dose_info: if is_female && age_years >= 15 && age_years <= 49 { "育齡公費 1 劑 / 自費 1-2 劑" } else { "自費 1-2 劑 (隔 28 天)" }.into(),
                timing_info: "育齡婦女/出國旅遊".into(),
                category: if is_female && age_years >= 15 && age_years <= 49 { "Routine".into() } else { "SelfPaid".into() },
                description: if is_female && age_years >= 15 && age_years <= 49 {
                    "15-49歲育齡婦女檢具近3個月德國麻疹抗體陰性證明者可公費接種1劑。其他人自費補打。"
                } else {
                    "1966年後出生或不具抗體者出國前建議自費補打 1-2 劑。"
                }.into(),
                audience: "Adults".into(),
            },
            VaccineItem {
                name: "M 痘疫苗 (Mpox Vaccine)".into(),
                dose_info: "共 2 劑 (間隔 4 週)".into(),
                timing_info: "具風險行為者".into(),
                category: "SelfPaid".into(),
                description: "暴露前/後預防與風險行為者自費/公費接種".into(),
                audience: "Adults".into(),
            },
        ];

        milestones_out.push(TimelineMilestone {
            title: "旅遊醫學與特定自費疫苗 (CDC 清單)".to_string(),
            age_months: total_months + 1,
            status: "Current".to_string(),
            vaccines: travel_specs,
        });
    }

    Ok(VaccineResponse {
        age_display,
        child_age_detail,
        gender_display,
        milestones: milestones_out,
    })
}

#[tauri::command]
fn get_all_vaccines() -> Vec<VaccineDetailDoc> {
    vec![
        VaccineDetailDoc {
            id: "dtap-hib-ipv".into(),
            name: "五合一疫苗 (DTaP-Hib-IPV)".into(),
            aliases: "白喉破傷風非細胞性百日咳、b型嗜血桿菌及不活化小兒麻痺混合疫苗".into(),
            category: "Routine".into(),
            target_audience: "兒童/幼兒".into(),
            prevent_disease: "白喉、破傷風、百日咳、b型嗜血桿菌侵襲性感染、小兒麻痺症".into(),
            full_description: "五合一疫苗為幼兒基礎免疫之核心疫苗，取代舊型全細胞百日咳疫苗，大幅降低發燒等不良反應。".into(),
            schedule: vec![
                "第 1 劑：出生滿 2 個月 (公費)".into(),
                "第 2 劑：出生滿 4 個月 (公費)".into(),
                "第 3 劑：出生滿 6 個月 (公費)".into(),
                "第 4 劑 (追加劑)：出生滿 18 個月 (公費)".into(),
            ],
            notes: "若接種後出現高燒不退或嚴重過敏反應，下一劑應由醫師評估。".into(),
        },
        VaccineDetailDoc {
            id: "hepb".into(),
            name: "B 型肝炎疫苗 (Hepatitis B)".into(),
            aliases: "B肝疫苗".into(),
            category: "Both".into(),
            target_audience: "全齡通用".into(),
            prevent_disease: "B 型肝炎病毒感染、慢性肝炎、肝硬化與肝癌".into(),
            full_description: "台灣自1984年推行新生兒B肝疫苗接種，顯著降低幼兒帶原率。成人檢驗無抗體者亦建議自費補打。".into(),
            schedule: vec![
                "第 1 劑：出生 24 小時內儘速接種 (公費)".into(),
                "第 2 劑：出生滿 1 個月 (公費)".into(),
                "第 3 劑：出生滿 6 個月 (公費)".into(),
                "成人補打：按 0, 1, 6 個月時程自費接種 3 劑".into(),
            ],
            notes: "高風險孕婦所生之新生兒，需於出生24小時內另外追加注射1劑B型肝炎免疫球蛋白(HBIG)。".into(),
        },
        VaccineDetailDoc {
            id: "bcg".into(),
            name: "卡介苗 (BCG)".into(),
            aliases: "結核病疫苗".into(),
            category: "Routine".into(),
            target_audience: "嬰幼兒".into(),
            prevent_disease: "結核性腦膜炎及結核病播散性感染".into(),
            full_description: "用於預防幼兒嚴重結核病。接種後局部會有小紅結節與膿疱，為正常免疫反應過程。".into(),
            schedule: vec![
                "單劑：建議於出生滿 5 至 8 個月時施打 (公費)".into(),
            ],
            notes: "若嬰幼兒有免疫缺損或疑似免疫缺陷家族史，嚴禁接種卡介苗。".into(),
        },
        VaccineDetailDoc {
            id: "pcv".into(),
            name: "結合型肺炎鏈球菌疫苗 (PCV13 / PCV20)".into(),
            aliases: "13價/20價肺炎鏈球菌疫苗".into(),
            category: "Both".into(),
            target_audience: "全齡通用 (幼兒與65歲以上長者)".into(),
            prevent_disease: "肺炎鏈球菌引發之肺炎、敗血症及腦膜炎性侵襲性疾病".into(),
            full_description: "提供強效抗原結合免疫記憶。公費提供幼兒3劑PCV13，以及65歲以上長者1劑PCV13/20銜接PPV23。".into(),
            schedule: vec![
                "幼兒第 1 劑：出生滿 2 個月 (公費)".into(),
                "幼兒第 2 劑：出生滿 4 個月 (公費)".into(),
                "幼兒第 3 劑 (追加)：出生滿 12-15 個月 (公費)".into(),
                "65歲以上長者：公費提供 1 劑 PCV13 / PCV20，間隔1年銜接 1 劑 PPV23".into(),
                "一般成人自費：建議高風險或50歲以上成人自費接種 1 劑 PCV20".into(),
            ],
            notes: "未滿 65 歲之高風險慢性病患者，建議諮詢醫師提前自費接種。".into(),
        },
        VaccineDetailDoc {
            id: "varicella".into(),
            name: "水痘疫苗 (Varicella)".into(),
            aliases: "水痘水泡疫苗".into(),
            category: "Both".into(),
            target_audience: "幼兒與無抗體青少年/成人".into(),
            prevent_disease: "水痘病毒感染、全身性水疱及繼發性細菌感染".into(),
            full_description: "活性減毒疫苗。公費補助1歲幼兒打第1劑，4-6歲小兒科醫學會強烈建議自費追加第2劑以達到完美保護力。".into(),
            schedule: vec![
                "幼兒第 1 劑：出生滿 12 個月 (公費)".into(),
                "幼兒第 2 劑：滿 4-6 歲 (建議自費追加)".into(),
                "13歲以上未曾感染且無抗體者：自費接種 2 劑 (間隔 4-8 週)".into(),
            ],
            notes: "孕婦及嚴重免疫抑制者禁忌接種活性減毒水痘疫苗。".into(),
        },
        VaccineDetailDoc {
            id: "mmr".into(),
            name: "麻疹腮腺炎德國麻疹混合疫苗 (MMR)".into(),
            aliases: "三合一MMR疫苗".into(),
            category: "Both".into(),
            target_audience: "全齡通用 (幼兒公費 / 育齡婦女公費 / 出國自費)".into(),
            prevent_disease: "麻疹、流行性腮腺炎、德國麻疹及先天性德國麻疹症候群 (CRS)".into(),
            full_description: "極高保護效力之活性減毒疫苗。育齡婦女若無德國麻疹抗體可公費接種。前往日本/東南亞前建議評估補打。".into(),
            schedule: vec![
                "第 1 劑：出生滿 12 個月 (公費)".into(),
                "第 2 劑：滿 5 歲至國小入學前 (公費)".into(),
                "15-49歲育齡婦女：檢具德國麻疹抗體陰性證明可公費接種 1 劑".into(),
                "1966年後出生/出國人員：可自費補打 1-2 劑 (間隔 28 天以上)".into(),
            ],
            notes: "接種後4週內應避免懷孕。孕婦禁忌接種。".into(),
        },
        VaccineDetailDoc {
            id: "je".into(),
            name: "日本腦炎疫苗 (JE)".into(),
            aliases: "日腦疫苗 (細胞培養活性減毒疫苗)".into(),
            category: "Both".into(),
            target_audience: "幼兒與成人高風險/旅遊".into(),
            prevent_disease: "日本腦炎病毒引發之急性腦炎與神經性後遺症".into(),
            full_description: "現行公費採用新型細胞培養活性減毒疫苗，僅需接種 2 劑即可產生長期免疫。".into(),
            schedule: vec![
                "第 1 劑：出生滿 15 個月 (公費)".into(),
                "第 2 劑：出生滿 27 個月 (公費，與第1劑隔12個月)".into(),
                "成人自費：經常於豬舍、水稻田周邊工作或旅遊者可自費補打 1 劑".into(),
            ],
            notes: "若有發燒或急性感染應延期接種。".into(),
        },
        VaccineDetailDoc {
            id: "hepa".into(),
            name: "A 型肝炎疫苗 (Hepatitis A)".into(),
            aliases: "A肝疫苗".into(),
            category: "Both".into(),
            target_audience: "幼兒公費 / 成人自費".into(),
            prevent_disease: "A 型肝炎病毒引起的急性肝炎、黃疸與急性肝衰竭".into(),
            full_description: "幼兒全面納入公費施打。成人慢性肝病患者、餐飲從業人員或前往流行地區者強烈建議自費接種。".into(),
            schedule: vec![
                "幼兒第 1 劑：出生滿 18 個月 (公費)".into(),
                "幼兒第 2 劑：出生滿 27 個月 (公費，隔6-12個月)".into(),
                "成人自費：按 0, 6-12 個月時程施打 2 劑 (自費)".into(),
            ],
            notes: "完整施打 2 劑後，免疫力可維持 20 年以上。".into(),
        },
        VaccineDetailDoc {
            id: "dtap-ipv".into(),
            name: "白喉破傷風百日咳及小兒麻痺疫苗 (DTaP-IPV 追加劑)".into(),
            aliases: "四合一疫苗 (入學前追加劑)".into(),
            category: "Routine".into(),
            target_audience: "滿 5 歲至國小入學前兒童".into(),
            prevent_disease: "白喉、破傷風、百日咳及小兒麻痺症".into(),
            full_description: "作為幼兒小學前之強化追加劑，維繫百日咳與小兒麻痺之社群抗體屏障。".into(),
            schedule: vec![
                "單劑 (第5劑)：滿 5 歲至國小入學前完成施打 (公費)".into(),
            ],
            notes: "國小入學查驗項目之一，建議於開學前完成。".into(),
        },
        VaccineDetailDoc {
            id: "rotavirus".into(),
            name: "輪狀病毒疫苗 (Rotavirus Vaccine)".into(),
            aliases: "輪狀口服疫苗".into(),
            category: "SelfPaid".into(),
            target_audience: "嬰幼兒 (2-8個月大)".into(),
            prevent_disease: "輪狀病毒引發之嬰幼兒嚴重嘔吐、水瀉、脫水與住院".into(),
            full_description: "口服活性減毒疫苗。分為2劑型(Rotateq/Rotarix)與3劑型，需於出生後8個月大前完成所有劑次。".into(),
            schedule: vec![
                "2劑型：出生滿 2 個月、4 個月各口服 1 劑 (自費)".into(),
                "3劑型：出生滿 2 個月、4 個月、6 個月各口服 1 劑 (自費)".into(),
            ],
            notes: "嬰兒若有腸套疊病史或腸道先天畸形者禁忌口服。".into(),
        },
        VaccineDetailDoc {
            id: "ev71".into(),
            name: "腸病毒 A71 型疫苗 (EV71 Vaccine)".into(),
            aliases: "腸病毒疫苗".into(),
            category: "SelfPaid".into(),
            target_audience: "滿2個月至6歲幼兒".into(),
            prevent_disease: "腸病毒 A71 型引發之手足口病、腦炎、肺水腫等重症與死亡".into(),
            full_description: "台灣國產研發非活性疫苗 (如高端、國光/安特羅)，專門針對最易引發重症的 A71 型病毒株。".into(),
            schedule: vec![
                "基礎劑第 1 劑：滿 2 個月至 6 歲以下 (自費)".into(),
                "基礎劑第 2 劑：與第 1 劑間隔 2 個月 (自費)".into(),
                "追加劑：依廠牌規定於 1 歲後追加 1 劑 (自費)".into(),
            ],
            notes: "本疫苗僅針對 A71 型，對其他型別腸病毒 (如克沙奇病毒) 無直接交叉保護。".into(),
        },
        VaccineDetailDoc {
            id: "hpv".into(),
            name: "人類乳突病毒疫苗 (HPV Vaccine)".into(),
            aliases: "子宮頸癌疫苗 / 九價 HPV 疫苗".into(),
            category: "Both".into(),
            target_audience: "國中女生(公費) / 9-45歲男女(自費)".into(),
            prevent_disease: "子宮頸癌、外陰癌、陰道癌、菜花 (尖形濕疣)、肛門癌及口咽癌".into(),
            full_description: "具備極高預防效果。國健署提供國中女生公費9價疫苗。男性接種亦能防範菜花與口咽癌風險。".into(),
            schedule: vec![
                "國中女生公費：2 劑 (按 0, 6 個月時程)".into(),
                "9-14歲男女自費：2 劑 (按 0, 6 個月時程)".into(),
                "15-45歲男女自費：3 劑 (按 0, 2, 6 個月時程)".into(),
            ],
            notes: "發燒或懷孕期間請延期接種。已發生性行為者接種仍具保護力。".into(),
        },
        VaccineDetailDoc {
            id: "shingles".into(),
            name: "非活性帶狀疱疹疫苗 (Shingrix)".into(),
            aliases: "皮蛇疫苗 / 欣安立適".into(),
            category: "SelfPaid".into(),
            target_audience: "50歲以上成人 / 18歲以上高風險對象".into(),
            prevent_disease: "帶狀疱疹 (皮蛇) 及長期疱疹後神經痛 (PHN)".into(),
            full_description: "基因重組非活性疫苗，保護力長達10年以上，防護率達90%-97%，優於舊型活性帶狀疱疹疫苗。".into(),
            schedule: vec![
                "第 1 劑：50歲以上或18歲以上免疫低下者 (自費)".into(),
                "第 2 劑：與第 1 劑間隔 2 至 6 個月 (自費)".into(),
            ],
            notes: "無論過去是否罹患過皮蛇，均建議接種。".into(),
        },
        VaccineDetailDoc {
            id: "flu".into(),
            name: "季節性流感疫苗 (Influenza Vaccine)".into(),
            aliases: "流感疫苗 (三價/四價/高劑量/細胞培養)".into(),
            category: "Both".into(),
            target_audience: "全齡通用 (6個月以上)".into(),
            prevent_disease: "季節性 A 型與 B 型流感及其引發之肺炎重症與死亡".into(),
            full_description: "每年秋冬開打。公費提供幼兒、長者、孕婦、慢性病患；非公費對象亦建議每年自費施打1劑。".into(),
            schedule: vec![
                "6個月以上至8歲初次接種：施打 2 劑 (間隔 4 週)".into(),
                "9歲以上及成人：每年接種 1 劑 (公費或自費)".into(),
            ],
            notes: "流感病毒變異快，必須每年重新接種最新病毒株之疫苗。".into(),
        },
        VaccineDetailDoc {
            id: "covid19".into(),
            name: "新冠病毒疫苗 (COVID-19 Vaccine)".into(),
            aliases: "COVID-19 疫苗 (JN.1 / 最新株)".into(),
            category: "Routine".into(),
            target_audience: "6個月大以上所有民眾".into(),
            prevent_disease: "新型冠狀病毒感染引發之重症、住院與死亡".into(),
            full_description: "依衛生福利部最新政策定期開打，建議每年秋冬與流感疫苗同時或間隔接種追加劑。".into(),
            schedule: vec![
                "滿6個月以上民眾：按最新衛福部規範接種 1-2 劑公費疫苗".into(),
            ],
            notes: "可與流感疫苗或肺炎鏈球菌疫苗同時在不同部位接種。".into(),
        },
        VaccineDetailDoc {
            id: "tdap".into(),
            name: "減量破傷風白喉百日咳疫苗 (Tdap)".into(),
            aliases: "Tdap 三合一疫苗 (成人型)".into(),
            category: "SelfPaid".into(),
            target_audience: "成人、孕婦及新手父母小兒照顧者".into(),
            prevent_disease: "成人與新生兒之百日咳、破傷風及白喉".into(),
            full_description: "成人百日咳抗體會隨時間衰退。建議所有成人每10年追加1劑。孕婦每胎次27-36週自費接種可保護新生兒。".into(),
            schedule: vec![
                "成人常規：建議每 10 年自費追加 1 劑".into(),
                "孕婦：每次懷孕第 27-36 週自費接種 1 劑".into(),
            ],
            notes: "產婦若產前未接種，應於生產後儘速補打 1 劑。".into(),
        },
        VaccineDetailDoc {
            id: "rsv".into(),
            name: "呼吸道細胞融合病毒疫苗 (RSV Vaccine)".into(),
            aliases: "RSV 疫苗 (Arexvy / Abrysvo)".into(),
            category: "SelfPaid".into(),
            target_audience: "60歲以上長者 / 孕婦(28-36週)".into(),
            prevent_disease: "RSV 引起的嚴重下呼吸道感染、肺炎、氣喘發作與住院".into(),
            full_description: "最新核准之蛋白重組疫苗。長者接種可防範重症；孕婦於懷孕後期接種可將抗體傳遞給嬰兒防護6個月。".into(),
            schedule: vec![
                "60歲以上長者：自費施打 1 劑".into(),
                "懷孕婦女：懷孕第 28-36 週自費施打 1 劑".into(),
            ],
            notes: "提供長者及無法直接打疫苗之新生兒絕佳被動免疫保護。".into(),
        },
        VaccineDetailDoc {
            id: "yellow-fever".into(),
            name: "黃熱病疫苗 (Yellow Fever Vaccine)".into(),
            aliases: "黃皮書疫苗 / 旅遊醫學門診".into(),
            category: "SelfPaid".into(),
            target_audience: "前往非洲、中南美洲流行地區旅客".into(),
            prevent_disease: "黃熱病病毒引發之急性黃疸、出血與肝腎衰竭".into(),
            full_description: "活性減毒疫苗。前往特定國家強制要求出示國際預防接種證明書 (黃皮書)，需於出國前10天於旅遊醫學門診施打。".into(),
            schedule: vec![
                "出國前 10 天於指定旅遊醫學門診自費施打 1 劑 (終生有效)".into(),
            ],
            notes: "蛋類嚴重過敏者、免疫缺損者或9個月以下嬰兒禁忌接種。".into(),
        },
        VaccineDetailDoc {
            id: "meningococcal".into(),
            name: "流行性腦脊髓膜炎疫苗 (Meningococcal Vaccine)".into(),
            aliases: "腦膜炎雙球菌疫苗 (MenACWY / MenB)".into(),
            category: "SelfPaid".into(),
            target_audience: "赴歐美留學生 / 沙烏地朝聖者 / 高風險工作".into(),
            prevent_disease: "腦膜炎雙球菌引發之劇烈頭痛、腦膜炎及爆發性敗血症".into(),
            full_description: "涵蓋 A, C, W, Y 型或 B 型。歐美多所大學強制入學前施打；赴麥加朝聖者亦為入境簽證強制規定。".into(),
            schedule: vec![
                "MenACWY (結合型)：自費接種 1 劑 (保護力約 5 年)".into(),
                "MenB (B型專用)：自費接種 2 劑 (間隔 1 個月)".into(),
            ],
            notes: "前往高風險宿舍環境前建議提前 2 週完成接種。".into(),
        },
        VaccineDetailDoc {
            id: "typhoid".into(),
            name: "傷寒疫苗 (Typhoid Vaccine)".into(),
            aliases: "傷寒多醣體疫苗".into(),
            category: "SelfPaid".into(),
            target_audience: "前往高風險衛生不良地區旅客".into(),
            prevent_disease: "傷寒桿菌引發之持續高燒、腹痛、腸出血與脾臟腫大".into(),
            full_description: "前往南亞 (如印度、尼泊爾)、東南亞衛生條件較差地區前建議至旅遊門診自費施打。".into(),
            schedule: vec![
                "出國前 2 週自費施打 1 劑 (保護效期為 3 年，持續高風險每3年追加)".into(),
            ],
            notes: "滿 2 歲以上方可接種本疫苗。".into(),
        },
        VaccineDetailDoc {
            id: "rabies".into(),
            name: "狂犬病疫苗 (Rabies Vaccine)".into(),
            aliases: "狂犬病不活化疫苗".into(),
            category: "SelfPaid".into(),
            target_audience: "暴露前預防 (獸醫/動物研究/旅遊) 或 暴露後處置".into(),
            prevent_disease: "狂犬病病毒引發之致命性急性腦脊髓炎 (致死率近100%)".into(),
            full_description: "不活化疫苗。前往狂犬病流行國野外活動或動物高風險接觸者建議預先完成 3 劑暴露前預防。".into(),
            schedule: vec![
                "暴露前預防 (自費)：按 0, 7, 21-28 天施打 3 劑".into(),
                "暴露後處置：遭疑似野生動物咬傷後，依醫囑於 0, 3, 7, 14 天補打 4 劑 (公費或自費)".into(),
            ],
            notes: "若遭咬傷應立即以肥皂及大量清水沖洗傷口15分鐘並儘速就醫。".into(),
        },
        VaccineDetailDoc {
            id: "mpox".into(),
            name: "M 痘疫苗 (Mpox Vaccine)".into(),
            aliases: "猴痘疫苗 (JYNNEOS)".into(),
            category: "Both".into(),
            target_audience: "高風險性行為對象 / 照顧者 / 暴露後處置".into(),
            prevent_disease: "M 痘病毒引發之發燒、淋巴腺腫大與全身皮膚水疱病變".into(),
            full_description: "採用第三代非複製型減毒痘病毒疫苗 (JYNNEOS)，安全性極高，副作用輕微。".into(),
            schedule: vec![
                "第 1 劑：符合衛福部條件者公費或高風險自費接種".into(),
                "第 2 劑：與第 1 劑間隔至少 4 週 (28天) 完成完整保護".into(),
            ],
            notes: "皮內注射或皮下注射均可，建議完成 2 劑以獲得最大防護力。".into(),
        },
    ]
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_eligible_vaccines,
            get_all_vaccines
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
