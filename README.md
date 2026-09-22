# Hermes Agent Foundry

> **Zero-Code, Modular, Algorithmic Agent Creation Platform powered by Hermes Agent Runtime.**  
> *Arsitektur & Hak Cipta: Bagas Cihuy (Bagas Saputra)*

Hermes Agent Foundry adalah platform web untuk merakit, mengonfigurasi, dan menjalankan Agen AI Otonom secara instan tanpa koding manual. Platform ini memadukan kemudahan antarmuka *no-code/low-code* dengan keandalan mesin eksekusi algoritmik deterministik yang ditenagai oleh **Hermes Agent**.

---

## Fitur Utama

- **Tri-Modal Agent Creation:**
  1. **Prompt-to-Agent:** Ketik instruksi dalam bahasa alami, sistem mengompilasi menjadi graf alur kerja (*Directed Acyclic Graph / DAG*).
  2. **Katalog Template Siap Pakai:** 1-klik deploy agen untuk riset pasar, pemantau server, review kode, ekstraksi dokumen, hingga kurator konten media sosial.
  3. **Modular Lego Block Canvas:** Susun blok-blok modul taktil (Trigger, Context, Reasoning, Toolkits, Guardrails, Output) tanpa menulis sebaris kode pun.
- **Eksekusi Algoritmik Murni:** Alur kerja dijalankan menggunakan algoritma *Topological Sort (Kahn's Algorithm)* dengan pencegahan dependensi melingkar (*cycle detection*).
- **Safety Circuit Breaker:** Sirkuit pemutus otomatis terhadap *runaway loops*, kebocoran token (*token bleeding*), dan kegagalan berulang (*thrashing*).
- **Efisiensi Memori (Strict RAM Discipline):** Menerapkan model *Ephemeral Worker* di mana proses agen hanya aktif saat ada tugas dan langsung merilis memori setelah selesai (*exit 0*), mematuhi batas RAM server $\le 9.0$ GB.
- **Doktrin Nol Emoji (Zero Emoji Rule):** Tampilan visual bersih berstandar Neo-Brutalisme Taktil / *Warm Paper & Obsidian*, bebas dari karakter grafis emoji.

---

## Struktur Direktori

```
hermes-agent-foundry/
├── docs/
│   └── PRD_HERMES_AGENT_FOUNDRY.md   # Cetak biru PRD arsitektur lengkap
├── modules/                          # Taksonomi modul komputasi
│   ├── triggers/                     # Cron, Webhook, Event
│   ├── context/                      # Vault, RAG, KV Store
│   ├── reasoning/                    # ReAct, Structured, Critic
│   ├── tools/                        # WebSearch, Terminal, Browser
│   ├── guardrails/                   # Schema Validator, Budget Gate
│   └── outputs/                      # Telegram, Webhook, File Exporter
├── templates/                        # Katalog agen siap pakai (JSON)
│   ├── market_sentinel.json
│   ├── server_sentinel.json
│   ├── code_reviewer.json
│   ├── social_autopilot.json
│   └── doc_action_items.json
├── src/
│   ├── compiler/                     # Topological sort & DAG validator
│   ├── engine/                       # Execution runner & ephemeral worker
│   ├── guardrails/                   # Circuit breaker & rate limiter
│   └── scheduler/                    # Bridge ke hermes cron scheduler
├── tests/
│   └── run_all_tests.js              # Pengujian unit matematis & algoritmik
├── public/                           # Antarmuka web taktil no-code
├── server.js                         # Gateway API Express
├── package.json
└── README.md
```

---

## Uji Algoritma & Validasi

Jalankan rangkaian tes logika graf dan pemutus sirkuit:

```bash
node tests/run_all_tests.js
```

---

## Lisensi & Atribusi

Hak cipta dan rancang bangun arsitektur mutlak milik **Bagas Cihuy (Bagas Saputra)**.  
Ditenagai oleh runtime **Hermes Agent**.
