const { invoke } = window.__TAURI__.tauri;

document.addEventListener('DOMContentLoaded', () => {
  const monthSelect = document.getElementById('month');
  const daySelect = document.getElementById('day');
  
  // Populate months
  for (let i = 1; i <= 12; i++) {
    const opt = document.createElement('option');
    opt.value = i;
    opt.textContent = i;
    monthSelect.appendChild(opt);
  }

  // Populate days
  for (let i = 1; i <= 31; i++) {
    const opt = document.createElement('option');
    opt.value = i;
    opt.textContent = i;
    daySelect.appendChild(opt);
  }

  const calendarToggle = document.getElementById('calendar-toggle');
  const calendarLabel = document.getElementById('calendar-label');
  const yearInput = document.getElementById('year');

  calendarToggle.addEventListener('change', (e) => {
    if (e.target.checked) {
      calendarLabel.textContent = '目前使用：民國';
      yearInput.placeholder = '如: 80';
    } else {
      calendarLabel.textContent = '目前使用：西元';
      yearInput.placeholder = '如: 1990';
    }
  });

  const form = document.getElementById('dob-form');
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const year = parseInt(yearInput.value);
    const month = parseInt(monthSelect.value);
    const day = parseInt(daySelect.value);
    const isRoc = calendarToggle.checked;

    try {
      const response = await invoke('get_eligible_vaccines', { year, month, day, isRoc });
      displayVaccines(response);
    } catch (error) {
      alert(`錯誤: ${error}`);
    }
  });
});

function displayVaccines(data) {
  const { age_display, milestones } = data;
  const resultsDiv = document.getElementById('results');
  const timelineContainer = document.getElementById('timeline-container');
  const ageBadge = document.getElementById('age-badge');

  ageBadge.textContent = `目前計算年齡：${age_display}`;
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
        const tagClass = v.category === 'Routine' ? 'routine' : 'high-risk';
        const tagText = v.category === 'Routine' ? '常規建議' : '高風險對象';

        cardsHtml += `
          <div class="timeline-vaccine-card">
            <div class="card-header">
              <h4>${v.name}</h4>
              <span class="tag ${tagClass}">${tagText}</span>
            </div>
            <div class="meta-badges">
              <span class="dose-badge">💉 ${v.dose_info}</span>
              <span class="timing-badge">📅 ${v.timing_info}</span>
            </div>
            <p class="dose-desc">${v.description}</p>
          </div>
        `;
      });

      node.innerHTML = `
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
}
