import re

with open(r"e:\Vaccine\vaccine-app\src\styles.css", "r", encoding="utf-8") as f:
    css = f.read()

# :root variables
css = re.sub(
    r":root \{[\s\S]*?\}",
    """:root {
  --bg-gradient: linear-gradient(135deg, #F5F0E8 0%, #EBF0F0 50%, #F0EBF5 100%);
  --card-bg: rgba(255, 253, 248, 0.96);
  --card-border: rgba(180, 170, 155, 0.3);
  --primary-color: #4A7C88;
  --primary-hover: #3A6270;
  --text-main: #2D3748;
  --text-muted: #718096;
  --accent-color: #5B728A;
  --routine-bg: #E8F0E9;
  --routine-color: #3D7A5A;
  --high-risk-bg: #F0E8E6;
  --high-risk-color: #8B4A42;
}""",
    css
)

# Header h1 gradient
css = css.replace(
    "background: linear-gradient(to right, #e11d48, #f43f5e, #6366f1);",
    "background: linear-gradient(to right, #4A7C88, #5B728A, #7A6A88);"
)

# Tab active
css = re.sub(
    r"\.tab-btn\.active\s*\{[\s\S]*?\}",
    """.tab-btn.active {
  background: linear-gradient(135deg, #4A7C88, #5B728A);
  color: white;
  border-color: transparent;
  box-shadow: 0 4px 15px rgba(74, 124, 136, 0.3);
}""",
    css
)

# Primary button
css = re.sub(
    r"\.btn-primary\s*\{[^}]*background:\s*linear-gradient[^}]*\}",
    lambda m: m.group(0).replace("linear-gradient(135deg, #f43f5e 0%, #e11d48 100%)", "linear-gradient(135deg, #4A7C88, #3A6270)").replace("rgba(244, 63, 94, 0.3)", "rgba(74, 124, 136, 0.3)"),
    css
)
css = css.replace("rgba(244, 63, 94, 0.45)", "rgba(74, 124, 136, 0.45)") # hover
css = css.replace("rgba(244, 63, 94, 0.3)", "rgba(74, 124, 136, 0.3)") # general

# Input focus
css = css.replace("box-shadow: 0 0 0 3px rgba(244, 63, 94, 0.2);", "box-shadow: 0 0 0 3px rgba(74, 124, 136, 0.15);")
css = css.replace("box-shadow: 0 0 10px rgba(244, 63, 94, 0.2);", "box-shadow: 0 0 10px rgba(74, 124, 136, 0.15);") # gender radio group

# Age badge
css = re.sub(
    r"\.age-badge\s*\{[\s\S]*?\}",
    """.age-badge {
  background: #E8EEF5;
  color: #3A5A80;
  padding: 0.4rem 0.85rem;
  border-radius: 20px;
  font-size: 0.95rem;
  font-weight: 600;
  border: 1px solid #A8C0D8;
}""",
    css
)

# Current timeline node
css = re.sub(
    r"\.current-node \.timeline-marker\s*\{[\s\S]*?\}",
    """.current-node .timeline-marker {
  border-color: #4A7C88;
  color: white;
  background: #4A7C88;
  box-shadow: 0 0 15px rgba(74, 124, 136, 0.5);
  animation: pulse-ring 2s infinite;
}""",
    css
)
css = css.replace("rgba(244, 63, 94, 0.6)", "rgba(74, 124, 136, 0.6)")
css = css.replace("rgba(244, 63, 94, 0)", "rgba(74, 124, 136, 0)")
css = css.replace("border: 2px solid #f43f5e;", "border: 2px solid #4A7C88;")
css = css.replace("box-shadow: 0 8px 30px rgba(244, 63, 94, 0.15);", "box-shadow: 0 8px 30px rgba(74, 124, 136, 0.15);")

# Visit schedule current card
css = re.sub(
    r"\.current-visit-card\s*\{[\s\S]*?\}",
    """.current-visit-card {
  background: linear-gradient(135deg, #F0EDE8 0%, #E8E5DF 100%);
  border: 1.5px solid #C8BFB0;
  box-shadow: 0 4px 15px rgba(74, 124, 136, 0.12);
}""",
    css
)
# Visit schedule next card
css = re.sub(
    r"\.next-visit-card\s*\{[\s\S]*?\}",
    """.next-visit-card {
  background: linear-gradient(135deg, #EBF0F0 0%, #E0EAEA 100%);
  border: 1.5px solid #AECDD1;
  box-shadow: 0 4px 15px rgba(91, 114, 138, 0.1);
}""",
    css
)

# Badges inside card
css = css.replace(
    ".current-badge-pill {\n  background: #f43f5e;\n  color: #ffffff;\n}",
    ".current-badge-pill {\n  background: #4A7C88;\n  color: #ffffff;\n}"
)
css = css.replace(
    ".next-badge-pill {\n  background: #0284c7;\n  color: #ffffff;\n}",
    ".next-badge-pill {\n  background: #5B728A;\n  color: #ffffff;\n}"
)

# Current-visit-bar
css = re.sub(
    r"\.current-visit-bar\s*\{[\s\S]*?\}",
    """.current-visit-bar {
  background: linear-gradient(90deg, #F0EDE8 0%, #E8E5DF 100%);
  border: 1px solid #C8BFB0;
  border-left: 4px solid #4A7C88;
}""",
    css
)
# Next-visit-bar
css = re.sub(
    r"\.next-visit-bar\s*\{[\s\S]*?\}",
    """.next-visit-bar {
  background: linear-gradient(90deg, #EBF0F0 0%, #E0EAEA 100%);
  border: 1px solid #AECDD1;
  border-left: 4px solid #5B728A;
}""",
    css
)

# Add-cal-btn
css = re.sub(
    r"\.add-cal-btn\s*\{[\s\S]*?\}",
    """.add-cal-btn {
  background: #E8EEF5;
  color: #3A5A80;
  border: 1px solid #A8C0D8;
  padding: 0.35rem 0.75rem;
  border-radius: 8px;
  font-size: 0.82rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}""",
    css
)
css = re.sub(
    r"\.add-cal-btn:hover\s*\{[\s\S]*?\}",
    """.add-cal-btn:hover {
  background: #4A7C88;
  color: #ffffff;
  box-shadow: 0 0 10px rgba(74, 124, 136, 0.3);
  transform: scale(1.03);
}""",
    css
)

# Standalone current age banner
css = re.sub(
    r"\.standalone-current-age-banner\s*\{[\s\S]*?\}",
    """.standalone-current-age-banner {
  background: linear-gradient(135deg, #4A7C88 0%, #3A6270 100%);
  border: 1.5px solid #4A7C88;
  color: #ffffff;
  padding: 0.6rem 1.25rem;
  border-radius: 12px;
  font-size: 1.05rem;
  font-weight: 700;
  margin-bottom: 0.85rem;
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  box-shadow: 0 4px 20px rgba(74, 124, 136, 0.3);
}""",
    css
)

# current-vaccine-card
css = css.replace("border-color: #f43f5e;", "border-color: #4A7C88;")
css = css.replace("box-shadow: 0 4px 15px rgba(244, 63, 94, 0.1);", "box-shadow: 0 4px 15px rgba(74, 124, 136, 0.1);")

# Status pills
css = re.sub(r"\.past-pill\s*\{[^}]*\}", ".past-pill { background: #F0EDE8; color: #8A8070; border: 1px solid #C8BFB0; }", css)
css = re.sub(r"\.current-pill\s*\{[^}]*\}", ".current-pill { background: #E0EAEB; color: #3A6270; border: 1px solid #8AB8C0; }", css)
css = re.sub(r"\.next-pill\s*\{[^}]*\}", ".next-pill { background: #E8EEF5; color: #3A5A80; border: 1px solid #A8C0D8; }", css)

# Tags
css = re.sub(r"\.tag\.routine\s*\{[\s\S]*?\}", ".tag.routine {\n  background: #E8F0E9;\n  color: #3D7A5A;\n  border: 1px solid #A8CDB5;\n}", css)
css = re.sub(r"\.tag\.subsidized\s*\{[\s\S]*?\}", ".tag.subsidized {\n  background: #E8EEF5;\n  color: #3A5A80;\n  border: 1px solid #A8C0D8;\n}", css)
css = re.sub(r"\.tag\.self-paid\s*\{[\s\S]*?\}", ".tag.self-paid {\n  background: #F5EFE0;\n  color: #7A6030;\n  border: 1px solid #D4BC88;\n}", css)
css = re.sub(r"\.tag\.both-cat\s*\{[\s\S]*?\}", ".tag.both-cat {\n  background: #EEE8F5;\n  color: #5A3A7A;\n  border: 1px solid #C0A8D8;\n}", css)
css = re.sub(r"\.tag\.high-risk\s*\{[\s\S]*?\}", ".tag.high-risk {\n  background: #F0E8E6;\n  color: #8B4A42;\n  border: 1px solid #D4A8A0;\n}", css)

# Co-admin-box
css = re.sub(r"\.co-admin-box\s*\{[\s\S]*?\}", ".co-admin-box {\n  margin-top: 1rem;\n  background: #EFF4F5;\n  border: 1.5px solid #B8CDD1;\n  border-radius: 10px;\n  padding: 0.85rem 1.15rem;\n}", css)
css = css.replace("color: #0369a1;", "color: #3A6270;")

# Chip active
css = re.sub(
    r"\.chip\.active\s*\{[\s\S]*?\}",
    """.chip.active {
  background: linear-gradient(135deg, #4A7C88 0%, #5B728A 100%);
  color: white;
  border-color: transparent;
  box-shadow: 0 3px 10px rgba(74, 124, 136, 0.25);
}""",
    css
)

# Library card hover
css = re.sub(
    r"\.library-card:hover\s*\{[\s\S]*?\}",
    """.library-card:hover {
  transform: translateY(-4px);
  border-color: #4A7C88;
  box-shadow: 0 10px 25px rgba(74, 124, 136, 0.12);
}""",
    css
)

# Btn-detail-open
css = re.sub(
    r"\.btn-detail-open\s*\{[\s\S]*?\}",
    """.btn-detail-open {
  background: #E8EEF5;
  border: 1px solid #A8C0D8;
  color: #3A5A80;
  padding: 0.5rem;
  border-radius: 8px;
  font-size: 0.85rem;
  font-weight: 600;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
}""",
    css
)
css = css.replace(
    ".library-card:hover .btn-detail-open {\n  background: #f43f5e;\n  color: white;\n  border-color: transparent;\n}",
    ".library-card:hover .btn-detail-open {\n  background: #4A7C88;\n  color: white;\n  border-color: transparent;\n}"
)

# Modal card bg
css = re.sub(
    r"\.modal-card\s*\{[\s\S]*?\}",
    """.modal-card {
  background: #FDFAF6;
  border: 1.5px solid #D4C9B8;
  border-radius: 20px;
  width: 100%;
  max-width: 650px;
  max-height: 85vh;
  overflow-y: auto;
  padding: 2rem;
  position: relative;
  box-shadow: 0 20px 50px rgba(74, 124, 136, 0.15);
}""",
    css
)

# Catchup status badges
css = re.sub(r"\.catchup-status-badge\.ready\s*\{[\s\S]*?\}", ".catchup-status-badge.ready {\n  background: #E8F0E9;\n  color: #3D7A5A;\n  border: 1px solid #A8CDB5;\n}", css)
css = re.sub(r"\.catchup-status-badge\.waiting\s*\{[\s\S]*?\}", ".catchup-status-badge.waiting {\n  background: #F5EFE0;\n  color: #7A6030;\n  border: 1px solid #D4BC88;\n}", css)

# Catchup date banner
css = re.sub(
    r"\.catchup-date-banner\s*\{[\s\S]*?\}",
    """.catchup-date-banner {
  background: linear-gradient(135deg, #F0EDE8 0%, #E8E5DF 100%);
  border: 1.5px solid #C8BFB0;
  border-radius: 12px;
  padding: 1.5rem;
  text-align: center;
  margin-bottom: 1.25rem;
}""",
    css
)
css = css.replace(
    ".catchup-date-banner .date-value {\n  font-size: 1.8rem;\n  font-weight: 700;\n  color: #e11d48;\n  letter-spacing: 0.05em;\n  margin-bottom: 0.75rem;\n}",
    ".catchup-date-banner .date-value {\n  font-size: 1.8rem;\n  font-weight: 700;\n  color: #4A7C88;\n  letter-spacing: 0.05em;\n  margin-bottom: 0.75rem;\n}"
)

# Travel header banner
css = re.sub(
    r"\.travel-header-banner\s*\{[\s\S]*?\}",
    """.travel-header-banner {
  background: linear-gradient(135deg, #EBF0F0 0%, #E0EAEA 100%);
  border: 1px solid #AECDD1;
  border-radius: 12px;
  padding: 1.25rem 1.5rem;
  margin-bottom: 1.5rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.75rem;
}""",
    css
)
css = css.replace(
    ".travel-header-banner h3 {\n  font-size: 1.35rem;\n  color: #be123c;\n}",
    ".travel-header-banner h3 {\n  font-size: 1.35rem;\n  color: #2D3748;\n}"
)

# Travel badges
css = css.replace(".mandatory-badge { background: #ffe4e6; color: #be123c; border: 1px solid #fecdd3; }", ".mandatory-badge { background: #F0E8E6; color: #8B4A42; border: 1px solid #D4A8A0; }")
css = css.replace(".recommended-badge { background: #fef3c7; color: #b45309; border: 1px solid #fde68a; }", ".recommended-badge { background: #F5EFE0; color: #7A6030; border: 1px solid #D4BC88; }")
css = css.replace(".booster-badge { background: #e0f2fe; color: #0369a1; border: 1px solid #7dd3fc; }", ".booster-badge { background: #E8EEF5; color: #3A5A80; border: 1px solid #A8C0D8; }")

# Travel section titles
css = css.replace(".travel-section-title.mandatory-title { color: #dc2626; }", ".travel-section-title.mandatory-title { color: #8B4A42; }")
css = css.replace(".travel-section-title.recommended-title { color: #d97706; }", ".travel-section-title.recommended-title { color: #7A6030; }")
css = css.replace(".travel-section-title.booster-title { color: #0284c7; }", ".travel-section-title.booster-title { color: #3A5A80; }")

# Schedule list li
css = re.sub(
    r"\.schedule-list li\s*\{[\s\S]*?\}",
    """.schedule-list li {
  font-size: 0.95rem;
  color: #1e293b;
  background: #F5F0E8;
  padding: 0.5rem 0.75rem;
  border-radius: 8px;
  border: 1px solid #D8D0C0;
}""",
    css
)

# Print-btn
css = re.sub(
    r"\.print-btn\s*\{[\s\S]*?\}",
    """.print-btn {
  font-size: 0.9rem;
  padding: 0.5rem 1rem;
  background: #E8EEF5;
  color: #3A5A80;
  border: 1px solid #A8C0D8;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
}""",
    css
)
css = css.replace(
    ".print-btn:hover {\n  background: #0284c7;\n  color: #ffffff;\n  box-shadow: 0 0 12px rgba(2, 132, 199, 0.3);\n}",
    ".print-btn:hover {\n  background: #4A7C88;\n  color: #ffffff;\n  box-shadow: 0 0 12px rgba(74, 124, 136, 0.3);\n}"
)

# Additional color replacements for remaining red/pink to teal/slate
# .search-card h2, .library-header-card h2 color
css = css.replace("color: #e11d48;", "color: #3A6270;")
# label checks
css = css.replace("background: #ffe4e6;", "background: #F0E8E6;")
css = css.replace("color: #be123c;", "color: #8B4A42;")

with open(r"e:\Vaccine\vaccine-app\src\styles.css", "w", encoding="utf-8") as f:
    f.write(css)

print("Done")
