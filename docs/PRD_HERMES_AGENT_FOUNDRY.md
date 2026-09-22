# PRODUCT REQUIREMENTS DOCUMENT (PRD)
# HERMES AGENT FOUNDRY :: BLUEPRINT ARSITEKTUR SISTEM

**Nama Produk:** Hermes Agent Foundry  
**Pencipta & Pemilik Hak Cipta:** Bagas Cihuy (Bagas Saputra)  
**Status Dokumen:** Master Architectural Blueprint (Definitive / Zero Revision Goal)  
**Versi:** 1.0.0-PROD-SPEC  
**Tanggal:** 21 September 2026  
**Klasifikasi:** Core Engineering Specification  

---

## 1. RINGKASAN EKSEKUTIF & FILOSOFI PRODUK

### 1.1 Visi
Hermes Agent Foundry adalah platform berbasis web untuk merakit, mengonfigurasi, dan menjalankan Agen AI Otonom secara instan tanpa koding manual. Platform ini memadukan kemudahan antarmuka no-code/low-code dengan keandalan mesin eksekusi algoritmik deterministik yang ditenagai langsung oleh **Hermes Agent Runtime**.

### 1.2 Filosofi Desain: "Algorithmic Rigor, No Fuzzy Traps"
Kebanyakan platform pembuat agen AI di pasar (seperti Dify, Flowise, atau Custom GPTs) memperlakukan agen sebagai sekadar "prompt raksasa" yang rentan halusinasi, looping tanpa akhir (*infinite loops*), dan boros biaya token. 

Hermes Agent Foundry membalik paradigma ini:
1. **Agen Adalah Mesin Keadaan Terarah (Directed Acyclic Graph / DAG):** Setiap agen tersusun atas modul-modul komputasi diskret yang memiliki kontrak masukan (*input schema*), prakondisi (*preconditions*), logika transisi (*state transition*), dan pascakondisi (*invariants*).
2. **LLM Sebagai Mesin Inferensi, Bukan Pengendali Mutlak:** LLM hanya dipanggil untuk tugas penalaran semantik, ekstraksi data ambigu, dan sintesis konten. Alur kerja, orkestrasi tool, kontrol keamanan, penjadwalan, dan mitigasi error dikendalikan 100% oleh algoritma deterministik kode mesin.
3. **Eksekusi Efemeral Hemat Sumber Daya (Strict RAM Discipline):** Agen tidak berjalan sebagai daemon idle yang memakan memori konstan. Agen adalah konfigurasi deklaratif (JSON/SQLite). Saat terpicu, Hermes mengalokasikan proses worker terisolasi, menuntaskan tugas, mencatat checkpoint state ke ledger, dan keluar (*exit 0*), menjaga pemakaian RAM server selalu berada di bawah kuota mutlak $\le 9.0$ GB.

---

## 2. TIGA METODE PEMBUATAN AGEN (TRI-MODAL AGENT CREATION)

Platform menyediakan 3 jalur pembuatan agen yang dapat saling dikonversi secara transparan (*interchangeable*):

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     TRI-MODAL AGENT CREATION ENGINE                     │
├───────────────────┬─────────────────────────┬───────────────────────────┤
│ Mode 1:           │ Mode 2:                 │ Mode 3:                   │
│ PROMPT-TO-AGENT   │ TEMPLATE-BASED CATALOG  │ MODULAR LEGO CANVAS       │
├───────────────────┼─────────────────────────┼───────────────────────────┤
│ Ketik deskripsi   │ Pilih dari 10+ katalog  │ Susun modul visual blok   │
│ bahasa alami. AI  │ siap pakai. Masukkan    │ drag-and-drop: Trigger,   │
│ mengompilasi jadi │ kredensial & variabel.  │ Knowledge, Logic, Tools,  │
│ spesifikasi DAG.  │ 1-klik deploy.          │ Guardrails, Output.       │
└───────────────────┴─────────────────────────┴───────────────────────────┘
```

### 2.1 Mode 1: Prompt-to-Agent (Natural Language Synthesis)
- **Alur Kerja Pengguna:** Pengguna mengetik kebutuhan dalam kotak input tunggal, contoh:
  > *"Buatkan agen yang setiap hari Senin jam 08:00 WIB meriset berita tren teknologi dari Reddit dan HackerNews, merangkum intisarinya, memvalidasi sumber berita agar anti-hoaks, lalu mengirimkan laporan 1 halaman ke grup Telegram tim."*
- **Mekanisme di Bawah Kap Mesin (Hermes Compiler):**
  1. *Requirement Extraction:* Mengidentifikasi Trigger (`Cron: 0 8 * * 1 @ Asia/Jakarta`), Knowledge Sources (`Web/Reddit/HN`), Processing Logic (`Summarization + Grounded Citations`), Guardrail (`Fact Check / Bias Gate`), dan Output Channel (`Telegram Delivery`).
  2. *Topology Synthesis:* Menghubungkan modul-modul yang dibutuhkan ke dalam struktur DAG terurut.
  3. *Auto-Configuration:* Mengisi skema konfigurasi parameter default yang aman.
  4. *Preview & Test:* Menampilkan visualisasi graf alur agen sebelum pengguna menekan tombol `[AKTIFKAN AGEN]`.

### 2.2 Mode 2: Template-Based Catalog (1-Click Deployment)
Platform menyediakan katalog agen arketipe teruji yang siap pakai:
1. **Sentinel Pemantau Server & Log:** Memantau metrik VPS, deteksi lonjakan RAM/CPU, dan pengingat reboot/restart container jika crash.
2. **Intelijen Kompetitor & Harga:** Merayapi website target secara terjadwal, mencatat fluktuasi harga, dan memberi sinyal jika ada penurunan harga signifikan.
3. **Kurator & Penerbit Konten Media Sosial:** Menyusun draf postingan multi-slide (Swiss Editorial / Carousel), melakukan audit visual otomatis, dan menjadwalkan publikasi.
4. **Asisten Tinjau Kode (Pre-Commit Code Reviewer):** Menganalisis git diff, mendeteksi kerentanan keamanan (OWASP), dan memberikan saran refaktor efisien.
5. **Ekstraktor Dokumen ke Action Items:** Mengurai PDF/kontrak rapat menjadi tiket tugas tervalidasi dengan tenggat waktu dan penanggung jawab.
6. **Agen Triase Pesan & Inbox Email:** Memilah email/chat Telegram masuk berdasarkan urgensi dan menyusun draf balasan profesional.

### 2.3 Mode 3: Modular Lego Canvas (No-Code Visual Assembler)
Pengguna dapat merakit agen dari nol (*from scratch*) atau memodifikasi hasil generate Mode 1 & 2 melalui panel kanvas modular visual:
- Tidak memerlukan penulisan sintaks koding.
- Setiap modul berbentuk blok kartu taktil dengan terminal input (kiri/atas) dan terminal output (kanan/bawah).
- Konektor antar-blok memvalidasi kesesuaian tipe data secara real-time (*type-safe wire connection*).

---

## 3. TAKSONOMI MODUL PERAKITAN AGEN

Sistem membagi kemampuan agen ke dalam 6 kategori modul independen:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      MODULAR COMPONENT ARCHITECTURE                     │
├─────────────────────────────────────────────────────────────────────────┤
│ [1. TRIGGER]      Cron Scheduler | Webhook | Event Inbound | Manual Run │
│       │                                                                 │
│ [2. CONTEXT]      Vault Knowledge | Vector RAG | Scratchpad | Memory    │
│       │                                                                 │
│ [3. REASONING]    ReAct Engine | HTN Planner | Multi-Agent Debate | GoT │
│       │                                                                 │
│ [4. TOOLKITS]     Browser Use | Terminal Exec | REST API | SQL Query    │
│       │                                                                 │
│ [5. GUARDRAIL]    Schema Validator | Budget Cap | PII Mask | Human Gate │
│       │                                                                 │
│ [6. OUTPUT]       Telegram | Webhook Post | File Exporter | TTS Audio   │
└─────────────────────────────────────────────────────────────────────────┘
```

### Modul 1: Triggers (Pemicu Eksekusi)
- `Trigger.Cron`: Penjadwalan waktu berulang berbasis standar cron UNIX (contoh: `every 30m`, `0 7 * * *`). Terhubung ke scheduler internal Hermes.
- `Trigger.Webhook`: Endpoint URL unik untuk menerima payload JSON dari webhook luar (GitHub event, Stripe, dsb).
- `Trigger.Message`: Menunggu pesan masuk dari bot Telegram atau WhatsApp dengan filter kata kunci atau intent.
- `Trigger.Manual`: Tombol picu langsung dari dashboard web untuk eksekusi on-demand.

### Modul 2: Context & Memory (Basis Pengetahuan)
- `Context.ObsidianVault`: Akses baca-tulis terisolasi ke direktori catatan Markdown lokal.
- `Context.SemanticRAG`: Pencarian semantik potongan dokumen menggunakan vektor embedding lokal.
- `Context.EphemeralMemory`: Buffer memori jangka pendek yang hanya hidup selama sesi eksekusi tugas.
- `Context.PersistentKV`: Key-Value store berbasis SQLite untuk menyimpan status inkremental antar siklus eksekusi.

### Modul 3: Reasoning & Cognitive Engine (Logika Berpikir)
- `Reasoning.DirectPrompt`: Inferensi satu langkah untuk tugas deterministik ringkas.
- `Reasoning.ReAct`: Pola Thought -> Action -> Observation berulang dengan batas iterasi maksimum.
- `Reasoning.HTN`: Dekomposisi tugas hierarkis menjadi sub-tugas atomik berurutan.
- `Reasoning.CriticDebate`: Evaluasi ganda di mana draf jawaban pertama dikritisi oleh persona pengawas sebelum disetujui.

### Modul 4: Toolkits & Capabilities (Alat Kerja Lapangan)
- `Tools.WebSearch`: Mesin pencari internet untuk mengumpulkan fakta terkini.
- `Tools.BrowserExec`: Browser Chromium otomatis (via Browser Use harness) untuk navigasi web dinamis, form filling, dan screenshot.
- `Tools.TerminalExec`: Eksekusi bash aman dalam direktori kerja terisolasi.
- `Tools.HttpFetch`: Pemanggilan API pihak ketiga (GET/POST/PUT/DELETE) dengan bearer token terenkripsi.
- `Tools.DataStore`: Kueri baca/tulis ke SQLite atau PostgreSQL.

### Modul 5: Guardrails & Policy Gates (Gerbang Keselamatan & Kendala)
- `Guard.SchemaValidation`: Memastikan output agen 100% mematuhi format JSON Schema yang ditentukan. Jika tidak lolos, otomatis menolak dan memicu perbaikan instan.
- `Guard.TokenBudget`: Batas plafon konsumsi token per run (contoh: max 15.000 token). Jika terlampaui, sirkuit pemutus (*circuit breaker*) langsung menghentikan agen untuk mencegah pemborosan biaya.
- `Guard.LoopBreaker`: Mendeteksi pengulangan aksi yang sama lebih dari 3 kali berturut-turut (*thrashing*).
- `Guard.HumanInTheLoop`: Menahan eksekusi aksi berbahaya (seperti transfer dana, hapus database, posting publik) sampai pengguna memberi persetujuan via tombol Telegram / Web.

### Modul 6: Outputs & Notification (Kanal Distribusi)
- `Output.Telegram`: Mengirim teks berformat, gambar, atau dokumen ke chat/grup Telegram via bot.
- `Output.WebhookDispatch`: Mengirim hasil eksekusi ke URL API eksternal.
- `Output.FileStorage`: Menyimpan berkas output ke filesystem lokal (`.md`, `.json`, `.csv`, `.png`).
- `Output.AudioVoice`: Mengonversi teks laporan menjadi pesan suara menggunakan model Text-to-Speech (TTS).

---

## 4. ANALISIS TANTANGAN MASA DEPAN & MITIGASI ARSITEKTUR

Membangun platform agen tanpa koding memiliki sejumlah jebakan teknis kritis. Berikut adalah identifikasi risiko beserta solusi arsitektur yang wajib terpasang sejak hari pertama:

```
┌─────────────────────────────────────────────────────────────────────────┐
│               RISK MATRIX & PRE-ENGINEERED MITIGATIONS                  │
├─────────────────────────┬───────────────────────────────────────────────┤
│ Potensi Risiko / Masalah│ Solusi & Mitigasi Arsitektur Hermes           │
├─────────────────────────┼───────────────────────────────────────────────┤
│ 1. Infinite Token Loop  │ • Circuit Breaker: Max-turn threshold (<=10)  │
│    & Biaya Membengkak   │ • Token Allocation Guard per langkah eksekusi │
│                         │ • Stop condition berbasis konvergensi semantik│
├─────────────────────────┼───────────────────────────────────────────────┤
│ 2. Tool Argument        │ • Strict JSON Schema validation via Zod/AJV   │
│    Hallucination        │ • Type casting & sanitasi sebelum dispatch    │
│                         │ • Zero-shot reflection saat schema mismatch   │
├─────────────────────────┼───────────────────────────────────────────────┤
│ 3. Server RAM Overload  │ • Ephemeral Worker Model: Proses mati usai run│
│    (Kuota Limit <= 9GB) │ • Pool size maksimal 2 worker aktif bersamaan │
│                         │ • SQLite WAL mode (zero idle memory overhead) │
├─────────────────────────┼───────────────────────────────────────────────┤
│ 4. Eksekusi Shell /     │ • Restricted directory root chroot/workspace  │
│    Tool Liar Tak Aman   │ • Blacklist perintah berbahaya (rm -rf, dd)   │
│                         │ • Read-only default; write butuh deklarasi    │
├─────────────────────────┼───────────────────────────────────────────────┤
│ 5. Kegagalan Jaringan & │ • Idempotency key per siklus eksekusi         │
│    State Putus di Tengah│ • Checkpointing state parsial ke SQLite       │
│                         │ • Exponential backoff retry (maks 3 kali)     │
├─────────────────────────┼───────────────────────────────────────────────┤
│ 6. Bocornya Kredensial  │ • Penyimpanan token dengan enkripsi AES-256   │
│    & API Key Pengguna   │ • Redaksi otomatis token pada log dan trace   │
│                         │ • Secret isolation per agent workspace        │
├─────────────────────────┼───────────────────────────────────────────────┤
│ 7. Zombie Process pada  │ • Process supervisor dengan hard-timeout      │
│    Cron Scheduler       │ • Orphan process cleaner saat health-check    │
│                         │ • Health-pulse endpoint per 60 detik          │
└─────────────────────────┴───────────────────────────────────────────────┘
```

### Rincian Mitigasi Risiko Kritis:

#### Risiko A: Kebocoran RAM & CPU (Server Throttling)
- **Tantangan:** Jika setiap agen berjalan sebagai daemon Node.js / Python persisten yang selalu hidup di background, 10 agen saja akan menghabiskan 2-3 GB RAM, melanggar batas ketat server ($\le 9.0$ GB).
- **Solusi Rekayasa:** Menggunakan model **Ephemeral On-Demand Worker**. Seluruh metadata agen disimpan di database SQLite ringan (`agents.sqlite`). Saat pemicu (jadwal cron atau webhook) tiba, engine hanya memanggil satu proses worker sementara (`hermes-worker --agent-id <id>`). Setelah menyelesaikan seluruh simpul DAG, worker merilis memori dan menutup proses (*zero footprint at rest*).

#### Risiko B: Halusinasi Parameter Alat Kerja
- **Tantangan:** LLM sering mengarang format tanggal, nama file, atau struktur JSON saat memanggil REST API atau tool sistem, menyebabkan error 400/500 atau data korup.
- **Solusi Rekayasa:** Penerapan **Deterministic Type Casting & Coercion**. Setiap alat memiliki spesifikasi skema tipe data ketat. Jika model mengembalikan parameter tidak sesuai (contoh: string alih-alih integer), layer runtime akan melakukan *auto-cast* otomatis atau melemparkan umpan balik koreksi struktural terarah (*targeted correction message*) tanpa mengulang dari awal.

#### Risiko C: Eksekusi Tak Terkendali (Runaway Agent)
- **Tantangan:** Agen mengalami kebingungan (*reasoning loop*) dan terus memanggil alat berulang kali tanpa menghasilkan kesimpulan.
- **Solusi Rekayasa:** Setiap agen memiliki parameter `max_execution_steps` (default: 8, max: 20) dan `execution_timeout_seconds` (default: 180s). Ketika ambang batas tercapai, sistem memutus eksekusi (*graceful abort*), mencatat status `TIMEOUT_ABORT` pada log audit, dan mengirimkan notifikasi peringatan kepada operator.

---

## 5. SKEMA SPESIFIKASI DATA (DECLARATIVE AGENT SPECIFICATION - DAS)

Setiap agen di Hermes Agent Foundry direpresentasikan dalam format deklaratif JSON/YAML terstandarisasi. Ini memastikan agen bersifat portabel, dapat di-*export/import*, dan di-*version control* dengan Git.

### 5.1 Struktur Skema Agen (`agent.spec.json`)

```json
{
  "$schema": "https://hermes-foundry.local/schemas/agent-spec.v1.json",
  "id": "agent-market-sentinel-01",
  "name": "Sentinel Tren Pasar Harian",
  "version": "1.0.0",
  "description": "Memantau berita ekonomi makro dan harga kripto setiap pagi dan merangkumnya ke Telegram.",
  "author": "Bagas Cihuy",
  "metadata": {
    "category": "FINANCE",
    "created_at": "2026-09-21T10:00:00Z",
    "tags": ["crypto", "finance", "telegram", "cron"]
  },
  "constraints": {
    "max_steps": 10,
    "timeout_seconds": 180,
    "token_budget_per_run": 12000,
    "requires_human_approval": false
  },
  "trigger": {
    "type": "cron",
    "config": {
      "schedule": "0 7 * * *",
      "timezone": "Asia/Jakarta"
    }
  },
  "modules": [
    {
      "id": "mod_search",
      "type": "tool",
      "provider": "builtin.web_search",
      "params": {
        "query_template": "crypto market recap and macro economic news {{date_today}}",
        "limit": 5
      }
    },
    {
      "id": "mod_synthesize",
      "type": "reasoning",
      "provider": "hermes.reasoning.structured",
      "params": {
        "model": "ag/claude-sonnet-4-6",
        "prompt_template": "Sintesiskan fakta berikut menjadi laporan padat 3 bagian: 1) Ringkasan Pasar, 2) Pergerakan Utama, 3) Sentimen Risiko. Sumber data: {{mod_search.output}}",
        "temperature": 0.2
      }
    },
    {
      "id": "mod_guardrail",
      "type": "guardrail",
      "provider": "hermes.guard.schema",
      "params": {
        "required_sections": ["Ringkasan Pasar", "Pergerakan Utama", "Sentimen Risiko"],
        "max_length_chars": 2000,
        "ban_words": ["dijamin untung", "pasti to the moon"]
      }
    },
    {
      "id": "mod_dispatch",
      "type": "output",
      "provider": "builtin.telegram",
      "params": {
        "target_chat_id": "-1004397580704",
        "message_template": "[[LAPORAN PASAR HARIAN]]\n\n{{mod_synthesize.output}}",
        "parse_mode": "Markdown"
      }
    }
  ],
  "pipeline_dag": {
    "nodes": ["mod_search", "mod_synthesize", "mod_guardrail", "mod_dispatch"],
    "edges": [
      {"from": "mod_search", "to": "mod_synthesize"},
      {"from": "mod_synthesize", "to": "mod_guardrail"},
      {"from": "mod_guardrail", "to": "mod_dispatch"}
    ]
  }
}
```

---

## 6. SPESIFIKASI ANTARMUKA PENGGUNA (UI/UX SPECIFICATION)

Antarmuka web Hermes Agent Foundry mengadopsi standar visual **Tactile Pastel Pop / Neo-Brutalisme Fungsional** yang konsisten dengan estetika jajandigital:
- **DOKTRIN NOL EMOJI (ZERO EMOJI RULE):** Dilarang keras menggunakan karakter emoji grafis. Seluruh penanda visual menggunakan badge teks kurung siku `[...]`, tipografi *monospace*, simbol ASCII geometri (`•`, `▼`, `->`, `+`), dan palet warna terkalibrasi.
- **Palet Warna:** 
  - Canvas: Warm Paper Milk (`#FDFBF7`)
  - Border: Deep Ink Shadow (`#1E1B4B`) tebal 1.5px - 2px dengan bayangan solid `2px 2px 0px #1E1B4B`.
  - Aksentil: Bubblegum Pink (`#F472B6`), Lavender Lilac (`#C084FC`), Mint Ice (`#34D399`), Soft Honey Butter (`#FBBF24`), dan Sky Blue (`#38BDF8`).
- **Tata Letak Responsif (Desktop & Mobile-First):**
  - **Layar Desktop:** Tampilan terbagi (*Split Studio*) dengan kanvas graf blok di sisi kiri dan panel konfigurasi parameter & simulasi uji run di sisi kanan.
  - **Layar Smartphone:** Bilah navigasi bawah (*Bottom Nav Dock*) ergonomis dengan 4 tab utama: `[KATALOG]`, `[KANVAS]`, `[LOG RUN]`, dan `[MONITOR]`. Elemen formulir modul dirancang dalam bentuk kartu akordeon taktil yang ramah sentuhan ibu jari.

---

## 7. STRUKTUR DIREKTORI PROYEK & REPOSITORI GITHUB

Proyek ini dipersiapkan secara independen di direktori:
`/home/ubuntu/hermes-agent-foundry`

Telah diinisialisasi dengan repositori Git terpusat yang diarahkan ke:
`git@github.com:Bagas10k/hermes-agent-foundry.git` (Branch: `main`).

### Pohon Struktur Berkas Arsitektural:

```
/home/ubuntu/hermes-agent-foundry/
├── .git/                           # Repositori Git lokal
├── .gitignore                      # Proteksi secret, SQLite DB, cache, dan node_modules
├── package.json                    # Dependensi Express, Better-SQLite3, Zod
├── README.md                       # Dokumentasi resmi repositori
├── server.js                       # Server gateway API & web server statis
├── data/                           # Basis data lokal
│   ├── foundry.sqlite              # Penyimpanan deklarasi agen, modul, dan template
│   └── runs.sqlite                 # Audit log eksekusi dan telemetri runtime
├── docs/                           # Dokumentasi & Spesifikasi
│   └── PRD_HERMES_AGENT_FOUNDRY.md # Cetak biru PRD lengkap
├── modules/                        # Katalog modul komputasi independen
│   ├── triggers/                   # Handler pemicu (cron, webhook, manual)
│   ├── context/                    # Handler memori & dokumen (vault, RAG, KV)
│   ├── reasoning/                  # Engine pemikir (direct, ReAct, critic)
│   ├── tools/                      # Tool eksekutor (web search, browser, terminal)
│   ├── guardrails/                 # Validator skema & sirkuit pemutus
│   └── outputs/                    # Adapter kanal keluar (Telegram, webhook, file)
├── templates/                      # Katalog agen siap pakai (JSON template)
│   ├── market_sentinel.json
│   ├── server_sentinel.json
│   ├── code_reviewer.json
│   ├── social_autopilot.json
│   └── doc_action_items.json
├── src/                            # Engine inti platform
│   ├── compiler/                   # Penerjemah prompt & visual blok ke DAG
│   │   ├── prompt_to_agent.js      # Generator deklarasi agen dari prompt
│   │   └── dag_validator.js        # Validasi ketiadaan siklus dan type-safety
│   ├── engine/                     # Runtime eksekutor Hermes
│   │   ├── dag_runner.js           # Eksekusi simpul DAG secara topologis
│   │   └── ephemeral_worker.js     # Runner worker sementara sekali jalan
│   ├── guardrails/                 # Validator kepatuhan skema & token
│   │   ├── schema_validator.js     # Validasi JSON Schema/Zod
│   │   └── circuit_breaker.js      # Deteksi loop tak hingga dan batas token
│   └── scheduler/                  # Penyelaras cron job Hermes
│       └── cron_bridge.js          # Sinkronisasi agen cron ke hermes cron scheduler
└── public/                         # Frontend Web App No-Code
    ├── index.html                  # Halaman utama kanvas no-code & katalog
    ├── css/
    │   └── foundry.css             # Tema Tactile Pastel Pop & Neo-Brutalism
    └── js/
        ├── app.js                  # Alur kerja UI & state management
        ├── visual_canvas.js        # Logika graf interaktif blok modular
        └── prompt_builder.js       # Asisten prompt-to-agent
```

---

## 8. STRATEGI IMPLEMENTASI & EKSEKUSI TERJADWAL

Pembangunan platform ini dibagi menjadi 4 tahap eksekusi modular yang siap dijadwalkan secara rutin:

### Fase 1: Core Engine & Declarative Specification (Target: Hari 1)
- Pembentukan basis data SQLite `foundry.sqlite` (tabel `agents`, `modules`, `templates`, `execution_logs`).
- Pembangunan `dag_validator.js` (algoritma *Topological Sort* & deteksi siklus).
- Implementasi `dag_runner.js` yang mengeksekusi modul-modul dasar secara deterministik.

### Fase 2: Katalog Modul Standar & Template Awal (Target: Hari 2)
- Registrasi 15 modul inti (Cron, Webhook, WebSearch, HTTP Fetch, ReAct Reasoning, Schema Guard, Telegram Dispatch).
- Pembuatan 5 file template agen siap pakai di direktori `/templates/`.
- Pengujian unit test eksekusi 1 template secara headless via CLI runner.

### Fase 3: Frontend Web Visual Canvas & Prompt Compiler (Target: Hari 3)
- Antarmuka web responsif di `/public/` dengan 3 mode (Prompt-to-Agent, Template Picker, Visual Block Canvas).
- Integrasi Hermes AI Prompt Compiler (`/api/compile-prompt`) yang mengubah instruksi teks menjadi JSON DAG siap pakai.
- Visualisasi alur graf blok taktil bebas emoji.

### Fase 4: Integrasi Hermes Runtime & Scheduler Bridge (Target: Hari 4)
- Integrasi modul `cron_bridge.js` ke sistem `hermes cron` server.
- Pengujian end-to-end: pembuatan agen dari web -> penjadwalan otomatis -> eksekusi worker efemeral -> pengiriman laporan ke grup Telegram -> pencatatan log audit.
- Push commit ke GitHub `git@github.com:Bagas10k/hermes-agent-foundry.git`.

---

## 9. KRITERIA KEBERHASILAN (DEFINITION OF DONE)

1. **Kecepatan Pembuatan Agen:** Pengguna awam dapat membuat dan mengaktifkan agen yang berfungsi penuh dalam waktu $< 60$ detik (via Prompt atau Template).
2. **Nol Halusinasi Eksekusi Tool:** 100% parameter tool divalidasi oleh skema tipe deterministik sebelum dipanggil.
3. **Efisiensi Memori (Zero Idle Overhead):** Pembuatan 50 agen baru tidak menambah pemakaian RAM server lebih dari 50 MB saat kondisi pasif/idle.
4. **Resistensi Terhadap Infinite Loop:** Sirkuit pemutus berhasil memutus eksekusi jika agen terjebak loop $> 10$ turn tanpa output valid.
5. **Kepatuhan Desain:** 100% bebas dari emoji, berpenampilan taktil pastel neo-brutalisme, dan berfungsi mulus di layar smartphone maupun desktop.
