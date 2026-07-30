const { invoke } = window.__TAURI__.tauri;

let allVaccinesList = [];
let currentFilter = 'all';

window.addEventListener('DOMContentLoaded', () => {
  setupDateSelectors();
  setupCalendarToggle();
  setupFormSubmit();
  setupTabs();
  setupLibraryFilterAndSearch();
  setupModalEvents();

  // 預先載入疫苗圖鑑庫
  loadVaccineLibrary();
});

function setupDateSelectors() {
  const yearInput = document.getElementById('year');
  const monthSelect = document.getElementById('month');
  const daySelect = document.getElementById('day');

  const now = new Date();
  yearInput.value = now.getFullYear();

  for (let m = 1; m <= 12; m++) {
    const opt = document.createElement('option');
    opt.value = m;
    opt.textContent = `${m} 月`;
    if (m === now.getMonth() + 1) opt.selected = true;
    monthSelect.appendChild(opt);
  }

  function updateDays() {
    const year = parseInt(yearInput.value) || 2000;
    const month = parseInt(monthSelect.value) || 1;
    const isRoc = document.getElementById('calendar-toggle').checked;
    const actualYear = isRoc ? year + 1911 : year;

    const daysInMonth = new Date(actualYear, month, 0).getDate();
    const currentSelectedDay = parseInt(daySelect.value) || now.getDate();

    daySelect.innerHTML = '';
    for (let d = 1; d <= daysInMonth; d++) {
      const opt = document.createElement('option');
      opt.value = d;
      opt.textContent = `${d} 日`;
      if (d === currentSelectedDay || (d === daysInMonth && currentSelectedDay > daysInMonth)) {
        opt.selected = true;
      }
      daySelect.appendChild(opt);
    }
  }

  yearInput.addEventListener('input', updateDays);
  monthSelect.addEventListener('change', updateDays);
  updateDays();
}

function setupCalendarToggle() {
  const toggle = document.getElementById('calendar-toggle');
  const text = document.getElementById('calendar-type-text');
  const yearInput = document.getElementById('year');

  toggle.addEventListener('change', (e) => {
    const now = new Date();
    if (e.target.checked) {
      text.textContent = '民國年 (ROC)';
      yearInput.placeholder = '如: 80';
      yearInput.value = now.getFullYear() - 1911;
    } else {
      text.textContent = '西元年 (AD)';
      yearInput.placeholder = '如: 1991';
      yearInput.value = now.getFullYear();
    }
    const event = new Event('input');
    yearInput.dispatchEvent(event);
  });
}

function setupFormSubmit() {
  const form = document.getElementById('dob-form');
  const yearInput = document.getElementById('year');
  const monthSelect = document.getElementById('month');
  const daySelect = document.getElementById('day');
  const calendarToggle = document.getElementById('calendar-toggle');

  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const year = parseInt(yearInput.value);
    const month = parseInt(monthSelect.value);
    const day = parseInt(daySelect.value);
    const isRoc = calendarToggle.checked;
    const gender = document.querySelector('input[name="gender"]:checked').value;

    try {
      const response = await invoke('get_eligible_vaccines', { year, month, day, isRoc, gender });
      displayVaccines(response);
    } catch (error) {
      alert(`錯誤: ${error}`);
    }
  });

  const jumpBtn = document.getElementById('jump-to-current');
  if (jumpBtn) {
    jumpBtn.addEventListener('click', scrollToCurrentNode);
  }
}

function scrollToCurrentNode() {
  const currentNode = document.querySelector('.current-node');
  if (currentNode) {
    currentNode.scrollIntoView({ behavior: 'smooth', block: 'center' });
  } else {
    const results = document.getElementById('results');
    if (results) {
      results.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }
}

function displayVaccines(data) {
  const { age_display, child_age_detail, gender_display, milestones } = data;
  const resultsDiv = document.getElementById('results');
  const timelineContainer = document.getElementById('timeline-container');
  const ageBadge = document.getElementById('age-badge');

  const ageText = child_age_detail || age_display;
  const fullMetaText = `${gender_display} | 目前計算年齡：${ageText}`;

  ageBadge.textContent = fullMetaText;
  timelineContainer.innerHTML = '';

  if (!milestones || milestones.length === 0) {
    timelineContainer.innerHTML = '<p class="empty" style="color: #94a3b8;">目前無特定時間軸資料。</p>';
  } else {
    milestones.forEach(m => {
      const node = document.createElement('div');
      
      let statusClass = 'next-node';
      let nodeIcon = '⏳';
      let statusLabel = '未來預計';

      if (m.status === 'Past') {
        statusClass = 'past-node';
        nodeIcon = '✓';
        statusLabel = '歷史已過期點';
      } else if (m.status === 'Current') {
        statusClass = 'current-node';
        nodeIcon = '📍';
        statusLabel = '當前推薦站點';
      }

      node.className = `timeline-item fade-in ${statusClass}`;

      let cardsHtml = '';
      m.vaccines.forEach(v => {
        let tagClass = 'routine';
        let tagText = '公費常規';
        if (v.category === 'SelfPaid') {
          tagClass = 'self-paid';
          tagText = '💰 自費建議';
        } else if (v.category === 'HighRisk') {
          tagClass = 'high-risk';
          tagText = '高風險對象';
        }

        const isCurrent = m.status === 'Current';
        const cardHighlightClass = isCurrent ? 'current-vaccine-card' : '';
        const currentBadge = isCurrent ? '<span class="due-now-badge">🔥 目前應施打</span>' : '';

        cardsHtml += `
          <div class="timeline-vaccine-card ${cardHighlightClass}">
            <div class="card-header">
              <h4>${v.name}</h4>
              <span class="tag ${tagClass}">${tagText}</span>
            </div>
            <div class="meta-badges">
              <span class="dose-badge">💉 ${v.dose_info}</span>
              <span class="timing-badge">📅 ${v.timing_info}</span>
              ${currentBadge}
            </div>
            <p class="dose-desc">${v.description}</p>
          </div>
        `;
      });

      const currentAgeBannerHtml = m.status === 'Current'
        ? `<div class="standalone-current-age-banner">📍 目前計算年齡：<strong>${child_age_detail || age_display}</strong></div>`
        : '';

      node.innerHTML = `
        ${currentAgeBannerHtml}
        <div class="timeline-marker">
          <div class="marker-dot">${nodeIcon}</div>
        </div>
        <div class="timeline-content">
          <div class="timeline-header">
            <h3 class="milestone-title">${m.title}</h3>
            <span class="status-pill ${statusClass}-pill">${statusLabel}</span>
          </div>
          <div class="timeline-cards-grid">
            ${cardsHtml}
          </div>
        </div>
      `;
      timelineContainer.appendChild(node);
    });
  }

  resultsDiv.classList.remove('hidden');
  setTimeout(scrollToCurrentNode, 200);
}

// ----------------------------------------------------
// 分頁 2：疫苗圖鑑庫 (Library View & Modal)
// ----------------------------------------------------
function setupTabs() {
  const tabBtns = document.querySelectorAll('.tab-btn');
  tabBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      tabBtns.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');

      const targetId = btn.getAttribute('data-target');
      document.querySelectorAll('.tab-page').forEach(page => {
        page.classList.add('hidden');
        page.classList.remove('active');
      });

      const activePage = document.getElementById(targetId);
      if (activePage) {
        activePage.classList.remove('hidden');
        activePage.classList.add('active');
      }
    });
  });
}

async function loadVaccineLibrary() {
  try {
    allVaccinesList = await invoke('get_all_vaccines');
    renderLibraryGrid(allVaccinesList);
  } catch (err) {
    console.error('載入疫苗圖鑑庫失敗:', err);
  }
}

function setupLibraryFilterAndSearch() {
  const searchInput = document.getElementById('library-search');
  const chips = document.querySelectorAll('.chip');

  if (searchInput) {
    searchInput.addEventListener('input', () => filterAndRenderLibrary());
  }

  chips.forEach(chip => {
    chip.addEventListener('click', () => {
      chips.forEach(c => c.classList.remove('active'));
      chip.classList.add('active');
      currentFilter = chip.getAttribute('data-filter');
      filterAndRenderLibrary();
    });
  });
}

function filterAndRenderLibrary() {
  const query = document.getElementById('library-search').value.toLowerCase().trim();

  const filtered = allVaccinesList.filter(item => {
    const matchesSearch = item.name.toLowerCase().includes(query) ||
                          item.aliases.toLowerCase().includes(query) ||
                          item.prevent_disease.toLowerCase().includes(query) ||
                          item.full_description.toLowerCase().includes(query);

    let matchesCategory = true;
    if (currentFilter === 'Routine') {
      matchesCategory = item.category === 'Routine' || item.category === 'Both';
    } else if (currentFilter === 'SelfPaid') {
      matchesCategory = item.category === 'SelfPaid' || item.category === 'Both';
    } else if (currentFilter === 'child') {
      matchesCategory = item.target_audience.includes('兒童') || item.target_audience.includes('全齡');
    } else if (currentFilter === 'adult') {
      matchesCategory = item.target_audience.includes('成人') || item.target_audience.includes('長者') || item.target_audience.includes('全齡');
    }

    return matchesSearch && matchesCategory;
  });

  renderLibraryGrid(filtered);
}

function renderLibraryGrid(vaccines) {
  const grid = document.getElementById('library-grid');
  if (!grid) return;

  grid.innerHTML = '';

  if (vaccines.length === 0) {
    grid.innerHTML = '<p class="empty" style="grid-column: 1/-1; text-align: center; color: #94a3b8; padding: 2rem;">未找到符合條件的疫苗。</p>';
    return;
  }

  vaccines.forEach(v => {
    const card = document.createElement('div');
    card.className = 'library-card fade-in';

    let catBadgeClass = 'routine';
    let catBadgeText = '公費常規';
    if (v.category === 'SelfPaid') {
      catBadgeClass = 'self-paid';
      catBadgeText = '💰 自費建議';
    } else if (v.category === 'Both') {
      catBadgeClass = 'both-cat';
      catBadgeText = '公費 / 自費';
    }

    card.innerHTML = `
      <div class="library-card-header">
        <span class="tag ${catBadgeClass}">${catBadgeText}</span>
        <span class="target-tag">🎯 ${v.target_audience}</span>
      </div>
      <h3 class="library-card-title">${v.name}</h3>
      <p class="library-aliases">${v.aliases}</p>
      <div class="prevent-disease-box">
        <strong>🛡️ 預防疾病：</strong>
        <p>${v.prevent_disease}</p>
      </div>
      <p class="library-desc-preview">${v.full_description.substring(0, 75)}...</p>
      <button class="btn-detail-open">查看完整介紹與注射時程 ➔</button>
    `;

    card.addEventListener('click', () => openVaccineModal(v));
    grid.appendChild(card);
  });
}

function openVaccineModal(v) {
  const modal = document.getElementById('vaccine-modal');
  const modalContent = document.getElementById('modal-content');

  let scheduleListHtml = '';
  v.schedule.forEach(s => {
    scheduleListHtml += `<li>💉 ${s}</li>`;
  });

  let catBadgeText = v.category === 'Routine' ? '公費常規' : (v.category === 'SelfPaid' ? '💰 自費建議' : '公費 / 自費提供');

  modalContent.innerHTML = `
    <div class="modal-header-section">
      <span class="modal-category-tag">${catBadgeText}</span>
      <h2>${v.name}</h2>
      <p class="modal-aliases">${v.aliases}</p>
    </div>

    <div class="modal-body-section">
      <div class="modal-block">
        <h3>🛡️ 預防疾病與感染</h3>
        <p class="disease-text">${v.prevent_disease}</p>
      </div>

      <div class="modal-block">
        <h3>📖 疫苗簡介與作用</h3>
        <p class="desc-text">${v.full_description}</p>
      </div>

      <div class="modal-block schedule-block">
        <h3>📅 建議注射時程與劑次</h3>
        <ul class="schedule-list">
          ${scheduleListHtml}
        </ul>
      </div>

      <div class="modal-block notes-block">
        <h3>⚠️ 接種注意事項與禁忌</h3>
        <p class="notes-text">${v.notes}</p>
      </div>
    </div>
  `;

  modal.classList.remove('hidden');
}

function setupModalEvents() {
  const modal = document.getElementById('vaccine-modal');
  const closeBtn = document.getElementById('modal-close');

  if (closeBtn) {
    closeBtn.addEventListener('click', () => modal.classList.add('hidden'));
  }

  if (modal) {
    modal.addEventListener('click', (e) => {
      if (e.target === modal) {
        modal.classList.add('hidden');
      }
    });
  }
}
