/**
 * visual_canvas.js
 * Visual Modular Canvas (Lego Block Builder) for Hermes Agent Foundry
 * Hak Cipta: Bagas Cihuy (Bagas Saputra)
 * 
 * Standar Desain:
 * - Tactile Pastel Pop & Warm Paper / Slate
 * - Zero-Slop, Zero-Emoji (ikon SVG monokrom & label mono presisi)
 * - Pure Vanilla JS (Zero external frontend dependencies, zero runtime bloat)
 * - Real-time Cycle Detection (Tarjan / DFS cycle check di browser)
 * - Type-checked connector cables (trigger, context, reasoning, tool, guardrail, output)
 * - Mobile Touch Support & Bottom Navigation Dock
 */

(function () {
  'use strict';

  // Palette Modul Lego (Cerah Terkalibrasi, Harmonis & Zero-Emoji)
  const MODULE_TYPES = {
    trigger: {
      label: 'TRIGGER',
      color: '#D97706', // Amber
      bg: '#FFFBEB',
      border: '#FDE68A',
      desc: 'Pemicu eksekusi (cron ticker, webhook inbound, manual)'
    },
    context: {
      label: 'CONTEXT',
      color: '#0284C7', // Sky Blue
      bg: '#F0F9FF',
      border: '#BAE6FD',
      desc: 'Injeksi konteks (Obsidian Vault, KV state persisten)'
    },
    reasoning: {
      label: 'REASONING',
      color: '#4F46E5', // Indigo
      bg: '#EEF2FF',
      border: '#C7D2FE',
      desc: 'Penalaran kognitif (ReAct loop, Hermes LLM Bridge)'
    },
    tool: {
      label: 'TOOL',
      color: '#059669', // Emerald
      bg: '#ECFDF5',
      border: '#A7F3D0',
      desc: 'Perkakas eksekusi (web search, terminal, browser, http)'
    },
    guardrail: {
      label: 'GUARDRAIL',
      color: '#DC2626', // Crimson
      bg: '#FEF2F2',
      border: '#FECACA',
      desc: 'Pengawal keamanan (CircuitBreaker, Critic, AST gate)'
    },
    output: {
      label: 'OUTPUT',
      color: '#7C3AED', // Violet
      bg: '#F5F3FF',
      border: '#DDD6FE',
      desc: 'Hasil luaran (Telegram alert, JSON response, Vault file)'
    }
  };

  // State Kanvas
  let nodes = [];
  let edges = [];
  let selectedNode = null;
  let connectingFrom = null; // node id saat sedang menarik kabel
  let isDraggingNode = false;
  let dragOffset = { x: 0, y: 0 };
  let canvasOffset = { x: 20, y: 20 };
  let canvasScale = 1.0;
  let isPanning = false;
  let panStart = { x: 0, y: 0 };

  // DOM Elements
  let containerEl, svgEl, nodesContainerEl, canvasToolbarEl;
  let validationBadgeEl, orderBadgeEl;

  // Inisialisasi Kanvas
  function initVisualCanvas(mountId) {
    containerEl = document.getElementById(mountId);
    if (!containerEl) return;

    containerEl.innerHTML = `
      <div class="canvas-wrapper" id="canvasWrapper">
        <div class="canvas-top-bar">
          <div class="canvas-brand-section">
            <span class="canvas-title">VISUAL MODULAR CANVAS</span>
            <span class="canvas-badge">LEGO BUILDER V1.0</span>
          </div>

          <div class="canvas-status-group">
            <div id="canvasValidationBadge" class="status-indicator valid">
              <span class="indicator-dot"></span>
              <span class="indicator-text">DAG VALID (0 SIKLUS)</span>
            </div>
            <div id="canvasOrderBadge" class="execution-order-pill">
              URUTAN: -
            </div>
          </div>

          <div class="canvas-action-group">
            <button id="btnAutoLayout" class="canvas-btn" title="Tata Otomatis Berdasarkan Lapisan (Sugiyama Hierarchical)">[TATA OTOMATIS]</button>
            <button id="btnClearCanvas" class="canvas-btn danger" title="Kosongkan Semua Simpul">[BERSIHKAN]</button>
            <button id="btnValidateDag" class="canvas-btn primary" title="Validasi & Hitung Toposort">[VERIFIKASI DAG]</button>
            <button id="btnDeployCanvas" class="canvas-btn success" title="Deploy Agen ke Engine Foundry">[DEPLOY KE FOUNDRY]</button>
          </div>
        </div>

        <div class="canvas-workspace" id="canvasWorkspace">
          <div class="canvas-grid-bg" id="canvasGridBg">
            <svg id="canvasSvg" class="connections-svg">
              <defs>
                <marker id="arrowhead" markerWidth="9" markerHeight="6" refX="8" refY="3" orient="auto">
                  <polygon points="0 0, 9 3, 0 6" fill="#64748B" />
                </marker>
                <marker id="arrowhead-active" markerWidth="9" markerHeight="6" refX="8" refY="3" orient="auto">
                  <polygon points="0 0, 9 3, 0 6" fill="#0F172A" />
                </marker>
                <marker id="arrowhead-error" markerWidth="9" markerHeight="6" refX="8" refY="3" orient="auto">
                  <polygon points="0 0, 9 3, 0 6" fill="#DC2626" />
                </marker>
              </defs>
              <g id="edgesGroup"></g>
              <path id="tempCable" class="temp-cable" d="" style="display:none;" />
            </svg>
            <div id="nodesContainer" class="nodes-container"></div>
          </div>

          <!-- Floating Palette Dock (Desktop & Mobile Responsive) -->
          <div class="floating-palette-dock" id="paletteDock">
            <div class="palette-header">TAMBAH MODUL LEGO</div>
            <div class="palette-buttons-row">
              <button class="palette-add-btn trigger" data-type="trigger">+ TRIGGER</button>
              <button class="palette-add-btn context" data-type="context">+ CONTEXT</button>
              <button class="palette-add-btn reasoning" data-type="reasoning">+ REASONING</button>
              <button class="palette-add-btn tool" data-type="tool">+ TOOL</button>
              <button class="palette-add-btn guardrail" data-type="guardrail">+ GUARDRAIL</button>
              <button class="palette-add-btn output" data-type="output">+ OUTPUT</button>
            </div>
          </div>

          <!-- Bottom Navigation Mobile Sheet for Quick Actions -->
          <div class="canvas-mobile-dock" id="mobileDock">
            <button class="mobile-dock-btn" id="mBtnAddNode">[+ MODUL]</button>
            <button class="mobile-dock-btn" id="mBtnAutoLayout">[TATA]</button>
            <button class="mobile-dock-btn" id="mBtnVerify">[VERIFIKASI]</button>
            <button class="mobile-dock-btn primary" id="mBtnDeploy">[DEPLOY]</button>
          </div>
        </div>
      </div>
    `;

    // Ikat referensi
    svgEl = document.getElementById('canvasSvg');
    nodesContainerEl = document.getElementById('nodesContainer');
    validationBadgeEl = document.getElementById('canvasValidationBadge');
    orderBadgeEl = document.getElementById('canvasOrderBadge');

    setupEventListeners();
    loadSampleDefaultDag();
  }

  // Event Listeners untuk Interaktivitas Mouse & Touch
  function setupEventListeners() {
    const workspace = document.getElementById('canvasWorkspace');

    // Pan Workspace (Mouse Drag on Background)
    workspace.addEventListener('mousedown', (e) => {
      if (e.target === workspace || e.target.id === 'canvasGridBg' || e.target.id === 'canvasSvg') {
        isPanning = true;
        panStart = { x: e.clientX - canvasOffset.x, y: e.clientY - canvasOffset.y };
        workspace.style.cursor = 'grabbing';
      }
    });

    window.addEventListener('mousemove', (e) => {
      if (isPanning) {
        canvasOffset.x = e.clientX - panStart.x;
        canvasOffset.y = e.clientY - panStart.y;
        updateWorkspaceTransform();
      } else if (isDraggingNode && selectedNode) {
        selectedNode.x = Math.max(10, (e.clientX - canvasOffset.x - dragOffset.x));
        selectedNode.y = Math.max(10, (e.clientY - canvasOffset.y - dragOffset.y));
        renderNodePosition(selectedNode);
        renderEdges();
      } else if (connectingFrom) {
        updateTempCable(e.clientX, e.clientY);
      }
    });

    window.addEventListener('mouseup', () => {
      if (isPanning) {
        isPanning = false;
        workspace.style.cursor = 'default';
      }
      if (isDraggingNode) {
        isDraggingNode = false;
        selectedNode = null;
      }
      if (connectingFrom) {
        connectingFrom = null;
        hideTempCable();
      }
    });

    // Touch Support for Mobile
    workspace.addEventListener('touchstart', (e) => {
      if (e.touches.length === 1 && (e.target === workspace || e.target.id === 'canvasGridBg' || e.target.id === 'canvasSvg')) {
        isPanning = true;
        panStart = { x: e.touches[0].clientX - canvasOffset.x, y: e.touches[0].clientY - canvasOffset.y };
      }
    }, { passive: true });

    window.addEventListener('touchmove', (e) => {
      if (isPanning && e.touches.length === 1) {
        canvasOffset.x = e.touches[0].clientX - panStart.x;
        canvasOffset.y = e.touches[0].clientY - panStart.y;
        updateWorkspaceTransform();
      } else if (isDraggingNode && selectedNode && e.touches.length === 1) {
        selectedNode.x = Math.max(10, (e.touches[0].clientX - canvasOffset.x - dragOffset.x));
        selectedNode.y = Math.max(10, (e.touches[0].clientY - canvasOffset.y - dragOffset.y));
        renderNodePosition(selectedNode);
        renderEdges();
      }
    }, { passive: true });

    window.addEventListener('touchend', () => {
      isPanning = false;
      isDraggingNode = false;
      selectedNode = null;
      if (connectingFrom) {
        connectingFrom = null;
        hideTempCable();
      }
    });

    // Palette Buttons
    document.querySelectorAll('.palette-add-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const type = btn.dataset.type;
        addNewModule(type);
      });
    });

    // Action Buttons
    document.getElementById('btnAutoLayout').addEventListener('click', autoLayoutNodes);
    document.getElementById('btnClearCanvas').addEventListener('click', clearAll);
    document.getElementById('btnValidateDag').addEventListener('click', () => {
      const res = validateCurrentDag();
      showToast(res.isValid ? '[DAG 100% VALID & BEBAS SIKLUS]' : `[GALAT: ${res.errors[0]}]`);
    });
    document.getElementById('btnDeployCanvas').addEventListener('click', deployCurrentCanvasAgent);

    // Mobile Dock Actions
    const mBtnAdd = document.getElementById('mBtnAddNode');
    if (mBtnAdd) {
      mBtnAdd.addEventListener('click', () => {
        const type = prompt('Pilih tipe modul:\n1. trigger\n2. context\n3. reasoning\n4. tool\n5. guardrail\n6. output', 'tool');
        if (type && MODULE_TYPES[type.toLowerCase()]) {
          addNewModule(type.toLowerCase());
        }
      });
    }
    const mBtnLayout = document.getElementById('mBtnAutoLayout');
    if (mBtnLayout) mBtnLayout.addEventListener('click', autoLayoutNodes);
    const mBtnVer = document.getElementById('mBtnVerify');
    if (mBtnVer) mBtnVer.addEventListener('click', () => {
      const res = validateCurrentDag();
      showToast(res.isValid ? '[DAG VALID]' : `[GALAT: ${res.errors[0]}]`);
    });
    const mBtnDep = document.getElementById('mBtnDeploy');
    if (mBtnDep) mBtnDep.addEventListener('click', deployCurrentCanvasAgent);
  }

  function updateWorkspaceTransform() {
    const grid = document.getElementById('canvasGridBg');
    if (grid) {
      grid.style.transform = `translate(${canvasOffset.x}px, ${canvasOffset.y}px) scale(${canvasScale})`;
    }
  }

  // Tambah Modul Baru
  function addNewModule(type, customName) {
    const meta = MODULE_TYPES[type] || MODULE_TYPES.tool;
    const id = `${type}_${Date.now().toString(36).slice(-4)}`;
    const name = customName || `${meta.label} - ${id.toUpperCase()}`;

    // Posisi default di tengah pandangan kanvas
    const x = Math.round(50 - canvasOffset.x + (nodes.length * 40) % 240);
    const y = Math.round(80 - canvasOffset.y + (nodes.length * 50) % 300);

    const node = {
      id,
      type,
      name,
      x,
      y,
      width: 220,
      height: 110,
      params: {}
    };

    nodes.push(node);
    renderNode(node);
    validateCurrentDag();
    renderEdges();
  }

  // Render Element Node ke DOM
  function renderNode(node) {
    const meta = MODULE_TYPES[node.type] || MODULE_TYPES.tool;
    const el = document.createElement('div');
    el.id = `node_${node.id}`;
    el.className = `canvas-node ${node.type}`;
    el.style.left = `${node.x}px`;
    el.style.top = `${node.y}px`;
    el.style.borderColor = meta.border;

    el.innerHTML = `
      <div class="node-header" style="background: ${meta.bg}; border-bottom: 1px solid ${meta.border};">
        <div class="node-badge" style="color: ${meta.color}; border-color: ${meta.border};">
          ${meta.label}
        </div>
        <button class="node-delete-btn" data-id="${node.id}" title="Hapus Modul">×</button>
      </div>
      <div class="node-body">
        <div class="node-title">${node.name}</div>
        <div class="node-id-mono">${node.id}</div>
      </div>
      
      <!-- Port Konektor In (Kiri) dan Out (Kanan) -->
      <div class="port port-in" data-id="${node.id}" data-port="in" title="Tautkan kabel masuk ke simpul ini">
        <div class="port-dot"></div>
      </div>
      <div class="port port-out" data-id="${node.id}" data-port="out" title="Tarik kabel keluar dari simpul ini">
        <div class="port-dot"></div>
      </div>
    `;

    // Dragging Handler
    el.addEventListener('mousedown', (e) => {
      if (e.target.classList.contains('port') || e.target.classList.contains('port-dot') || e.target.classList.contains('node-delete-btn')) {
        return;
      }
      isDraggingNode = true;
      selectedNode = node;
      dragOffset.x = e.clientX - canvasOffset.x - node.x;
      dragOffset.y = e.clientY - canvasOffset.y - node.y;
      document.querySelectorAll('.canvas-node').forEach(n => n.classList.remove('selected'));
      el.classList.add('selected');
      e.stopPropagation();
    });

    // Delete Button
    el.querySelector('.node-delete-btn').addEventListener('click', (e) => {
      e.stopPropagation();
      deleteNode(node.id);
    });

    // Port Out (Mulai Menarik Kabel)
    const portOut = el.querySelector('.port-out');
    portOut.addEventListener('mousedown', (e) => {
      e.stopPropagation();
      connectingFrom = node.id;
      showTempCable(node);
    });

    // Port In (Menyambungkan Kabel)
    const portIn = el.querySelector('.port-in');
    portIn.addEventListener('mouseup', (e) => {
      e.stopPropagation();
      if (connectingFrom && connectingFrom !== node.id) {
        addEdge(connectingFrom, node.id);
        connectingFrom = null;
        hideTempCable();
      }
    });

    nodesContainerEl.appendChild(el);
  }

  function renderNodePosition(node) {
    const el = document.getElementById(`node_${node.id}`);
    if (el) {
      el.style.left = `${node.x}px`;
      el.style.top = `${node.y}px`;
    }
  }

  function deleteNode(nodeId) {
    nodes = nodes.filter(n => n.id !== nodeId);
    edges = edges.filter(e => e.from !== nodeId && e.to !== nodeId);
    const el = document.getElementById(`node_${nodeId}`);
    if (el) el.remove();
    validateCurrentDag();
    renderEdges();
  }

  // Tambah Relasi Kabel (Edge) dengan Pemeriksaan Validitas
  function addEdge(fromId, toId) {
    if (fromId === toId) {
      showToast('[TIDAK BISA MENGHUBUNGKAN SIMPUL KE DIRINYA SENDIRI]');
      return;
    }

    // Cek duplikasi
    const exists = edges.some(e => e.from === fromId && e.to === toId);
    if (exists) {
      showToast('[KABEL RELASI SUDAH ADA]');
      return;
    }

    // Cek aturan tipe modular: Trigger tidak boleh menerima masukan
    const toNode = nodes.find(n => n.id === toId);
    if (toNode && toNode.type === 'trigger') {
      showToast('[MODUL TRIGGER MERUPAKAN SUMBER HULU, TIDAK BISA MENERIMA KABEL MASUK]');
      return;
    }

    // Tambah sementara dan uji siklus (Cycle Detection)
    edges.push({ from: fromId, to: toId });
    const validation = validateCurrentDag();

    if (!validation.isValid) {
      // Gulung balik jika memicu siklus melingkar
      edges.pop();
      showToast(`[KONEKSI DITOLAK: ${validation.errors[0]}]`);
      validateCurrentDag();
      renderEdges();
      return;
    }

    renderEdges();
    showToast('[KONEKSI KABEL BERHASIL DITAUTKAN]');
  }

  // Render Garis Hubung Kabel SVG (Cubic Bezier Curves ala n8n)
  function renderEdges() {
    const group = document.getElementById('edgesGroup');
    if (!group) return;
    group.innerHTML = '';

    const validation = validateCurrentDag();

    edges.forEach((edge, idx) => {
      const fromNode = nodes.find(n => n.id === edge.from);
      const toNode = nodes.find(n => n.id === edge.to);
      if (!fromNode || !toNode) return;

      const x1 = fromNode.x + 220; // sisi kanan
      const y1 = fromNode.y + 55;  // tengah tinggi
      const x2 = toNode.x;         // sisi kiri
      const y2 = toNode.y + 55;

      const dx = Math.abs(x2 - x1) * 0.5;
      const pathD = `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;

      const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
      path.setAttribute('d', pathD);
      path.setAttribute('class', 'cable-path');
      path.setAttribute('marker-end', 'url(#arrowhead)');
      path.setAttribute('data-index', idx);

      // Klik kabel untuk menghapus
      path.addEventListener('click', (e) => {
        e.stopPropagation();
        if (confirm(`Hapus koneksi kabel dari '${edge.from}' ke '${edge.to}'?`)) {
          edges.splice(idx, 1);
          validateCurrentDag();
          renderEdges();
        }
      });

      group.appendChild(path);
    });
  }

  // Temp Cable saat user menarik konektor
  function showTempCable(fromNode) {
    const temp = document.getElementById('tempCable');
    if (temp) {
      temp.style.display = 'block';
    }
  }

  function updateTempCable(clientX, clientY) {
    const fromNode = nodes.find(n => n.id === connectingFrom);
    if (!fromNode) return;

    const x1 = fromNode.x + 220;
    const y1 = fromNode.y + 55;
    const x2 = clientX - canvasOffset.x;
    const y2 = clientY - canvasOffset.y;

    const dx = Math.abs(x2 - x1) * 0.5;
    const pathD = `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;

    const temp = document.getElementById('tempCable');
    if (temp) {
      temp.setAttribute('d', pathD);
    }
  }

  function hideTempCable() {
    const temp = document.getElementById('tempCable');
    if (temp) {
      temp.style.display = 'none';
      temp.setAttribute('d', '');
    }
  }

  // Live DAG Validation & Topological Sort (Kahn's Algorithm di Browser)
  function validateCurrentDag() {
    const nodeIds = nodes.map(n => n.id);
    const inDegree = {};
    const adj = {};

    nodeIds.forEach(id => {
      inDegree[id] = 0;
      adj[id] = [];
    });

    const errors = [];

    // Periksa keabsahan edge
    edges.forEach(edge => {
      if (!inDegree.hasOwnProperty(edge.from) || !inDegree.hasOwnProperty(edge.to)) {
        errors.push(`Kabel merujuk ke simpul yang sudah terhapus`);
        return;
      }
      if (edge.from === edge.to) {
        errors.push(`Self-loop terdeteksi pada '${edge.from}'`);
        return;
      }
      adj[edge.from].push(edge.to);
      inDegree[edge.to] = (inDegree[edge.to] || 0) + 1;
    });

    if (errors.length > 0) {
      updateValidationUI(false, [], errors);
      return { isValid: false, executionOrder: [], errors };
    }

    if (nodes.length === 0) {
      updateValidationUI(true, [], []);
      return { isValid: true, executionOrder: [], errors: [] };
    }

    // Kahn's Algorithm
    const queue = [];
    nodeIds.forEach(id => {
      if (inDegree[id] === 0) {
        queue.push(id);
      }
    });

    const executionOrder = [];

    while (queue.length > 0) {
      const u = queue.shift();
      executionOrder.push(u);

      adj[u].forEach(v => {
        inDegree[v]--;
        if (inDegree[v] === 0) {
          queue.push(v);
        }
      });
    }

    if (executionOrder.length !== nodeIds.length) {
      const cycleError = `Mendeteksi ketergantungan melingkar (deadlock cycle) pada alur kabel!`;
      updateValidationUI(false, [], [cycleError]);
      return { isValid: false, executionOrder: [], errors: [cycleError] };
    }

    updateValidationUI(true, executionOrder, []);
    return { isValid: true, executionOrder, errors: [] };
  }

  function updateValidationUI(isValid, order, errors) {
    if (!validationBadgeEl || !orderBadgeEl) return;

    if (isValid) {
      validationBadgeEl.className = 'status-indicator valid';
      validationBadgeEl.innerHTML = `
        <span class="indicator-dot"></span>
        <span class="indicator-text">DAG VALID (BEBAS SIKLUS)</span>
      `;
      orderBadgeEl.textContent = order.length ? `URUTAN: ${order.join(' → ')}` : 'URUTAN: KOSONG';
    } else {
      validationBadgeEl.className = 'status-indicator error';
      validationBadgeEl.innerHTML = `
        <span class="indicator-dot error"></span>
        <span class="indicator-text">${errors[0] || 'GALAT SIKLUS'}</span>
      `;
      orderBadgeEl.textContent = 'URUTAN: TIDAK BISA DIHITUNG (DEADLOCK)';
    }
  }

  // Sugiyama Hierarchical Auto-Layout (Pemisahan Kolom Berdasarkan In-Degree / Layer)
  function autoLayoutNodes() {
    if (nodes.length === 0) return;

    const validation = validateCurrentDag();
    if (!validation.isValid) {
      showToast('[TIDAK BISA MENATA OTOMATIS: DAG MASIH MEMILIKI SIKLUS]');
      return;
    }

    // Hitung lapisan level tiap simpul (Longest Path Layering)
    const layers = {};
    nodes.forEach(n => layers[n.id] = 0);

    // Iterasi relasi untuk menentukan kedalaman layer
    for (let iter = 0; iter < nodes.length; iter++) {
      edges.forEach(e => {
        if (layers[e.to] <= layers[e.from]) {
          layers[e.to] = layers[e.from] + 1;
        }
      });
    }

    // Kelompokkan per layer
    const layerBuckets = {};
    nodes.forEach(n => {
      const lvl = layers[n.id] || 0;
      if (!layerBuckets[lvl]) layerBuckets[lvl] = [];
      layerBuckets[lvl].push(n);
    });

    const startX = 60;
    const startY = 60;
    const colSpacing = 300;
    const rowSpacing = 150;

    Object.keys(layerBuckets).forEach(lvl => {
      const bucket = layerBuckets[lvl];
      bucket.forEach((node, rowIdx) => {
        node.x = startX + (parseInt(lvl, 10) * colSpacing);
        node.y = startY + (rowIdx * rowSpacing);
        renderNodePosition(node);
      });
    });

    renderEdges();
    showToast('[TATA LETAK HIERARKIS RUST DAG BERHASIL DIRAPIKAN]');
  }

  function clearAll() {
    if (confirm('Kosongkan semua simpul dan kabel di kanvas?')) {
      nodes = [];
      edges = [];
      nodesContainerEl.innerHTML = '';
      const group = document.getElementById('edgesGroup');
      if (group) group.innerHTML = '';
      validateCurrentDag();
      showToast('[KANVAS TELAH DIBERSIHKAN]');
    }
  }

  // Load Dag Contoh (Bawaan Sentinel System)
  function loadSampleDefaultDag() {
    nodes = [
      { id: 'cron_ticker', type: 'trigger', name: 'Cron Ticker (15m)', x: 60, y: 120, width: 220, height: 110 },
      { id: 'vault_reader', type: 'context', name: 'Obsidian Vault Reader', x: 360, y: 40, width: 220, height: 110 },
      { id: 'react_core', type: 'reasoning', name: 'Hermes ReAct Engine', x: 660, y: 120, width: 220, height: 110 },
      { id: 'terminal_tool', type: 'tool', name: 'Terminal Shell Exec', x: 960, y: 40, width: 220, height: 110 },
      { id: 'critic_gate', type: 'guardrail', name: 'Critic & CircuitBreaker', x: 960, y: 200, width: 220, height: 110 },
      { id: 'telegram_out', type: 'output', name: 'Telegram Group BAgent', x: 1260, y: 120, width: 220, height: 110 }
    ];

    edges = [
      { from: 'cron_ticker', to: 'vault_reader' },
      { from: 'vault_reader', to: 'react_core' },
      { from: 'react_core', to: 'terminal_tool' },
      { from: 'react_core', to: 'critic_gate' },
      { from: 'terminal_tool', to: 'telegram_out' },
      { from: 'critic_gate', to: 'telegram_out' }
    ];

    nodes.forEach(n => renderNode(n));
    validateCurrentDag();
    renderEdges();
  }

  // Deploy Agen dari Kanvas ke Backend Foundry
  async function deployCurrentCanvasAgent() {
    const val = validateCurrentDag();
    if (!val.isValid) {
      showToast(`[TIDAK BISA MENYIMPAN: ${val.errors[0]}]`);
      return;
    }

    if (nodes.length === 0) {
      showToast('[KANVAS KOSONG, TIDAK ADA MODUL UNTUK DIDEPLOY]');
      return;
    }

    const agentName = prompt('Beri nama Agen Foundry Anda:', 'Agen Rakitan Kanvas Visual');
    if (!agentName) return;

    const agentId = 'agent_' + Date.now().toString(36);
    const agentSpec = {
      id: agentId,
      name: agentName,
      version: '1.0.0',
      category: 'CANVAS_BUILDER',
      description: `Agen modular rakitan visual via Visual Canvas Lego Builder (${nodes.length} modul).`,
      author: 'Bagas Cihuy',
      constraints: {
        max_steps: 10,
        timeout_seconds: 180,
        token_budget_per_run: 15000,
        requires_human_approval: false
      },
      trigger: {
        type: nodes.some(n => n.type === 'trigger') ? 'webhook' : 'manual',
        path: `/api/agents/${agentId}/trigger`
      },
      modules: nodes.map(n => ({
        id: n.id,
        type: n.type,
        provider: 'hermes-native',
        name: n.name,
        params: {}
      })),
      pipeline_dag: {
        nodes: nodes.map(n => n.id),
        edges: edges.map(e => ({ from: e.from, to: e.to }))
      }
    };

    try {
      showToast('[MENGIRIM AGEN KE ENGINE FOUNDRY...]');
      const res = await fetch('/api/agents', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(agentSpec)
      });
      const data = await res.json();
      if (data.success) {
        showToast(`[SUKSES: AGEN '${agentName}' AKTIF!]`);
        if (window.loadCatalog) {
          window.loadCatalog();
        }
      } else {
        showToast(`[GAGAL: ${data.error || 'Server menolak spec'}]`);
      }
    } catch (err) {
      showToast(`[GALAT: ${err.message}]`);
    }
  }

  // Ekspor Global
  window.initVisualCanvas = initVisualCanvas;

})();
