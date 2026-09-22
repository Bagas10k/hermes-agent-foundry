/**
 * Hermes Agent Foundry :: Mathematical & Algorithmic Unit Tests
 */

const { validateAndSortDAG, validateDataBindings } = require('../src/compiler/dag_validator');
const { CircuitBreaker } = require('../src/guardrails/circuit_breaker');

let passedTests = 0;
let failedTests = 0;

function assert(condition, message) {
  if (!condition) {
    console.error(`[FAIL] ${message}`);
    failedTests++;
  } else {
    console.log(`[PASS] ${message}`);
    passedTests++;
  }
}

console.log('=== MENJALANKAN UJI ALGORITMA HERMES AGENT FOUNDRY ===\n');

// ─── TEST 1: Linear Pipeline DAG ───
{
  const linearDag = {
    nodes: ['step_a', 'step_b', 'step_c'],
    edges: [
      { from: 'step_a', to: 'step_b' },
      { from: 'step_b', to: 'step_c' }
    ]
  };
  const res = validateAndSortDAG(linearDag);
  assert(res.isValid === true, 'Linear DAG terbukti valid');
  assert(JSON.stringify(res.executionOrder) === JSON.stringify(['step_a', 'step_b', 'step_c']), 'Urutan eksekusi linier presisi');
}

// ─── TEST 2: Diamond / Branching DAG ───
{
  const diamondDag = {
    nodes: ['root', 'branch_left', 'branch_right', 'join'],
    edges: [
      { from: 'root', to: 'branch_left' },
      { from: 'root', to: 'branch_right' },
      { from: 'branch_left', to: 'join' },
      { from: 'branch_right', to: 'join' }
    ]
  };
  const res = validateAndSortDAG(diamondDag);
  assert(res.isValid === true, 'Diamond DAG terbukti valid');
  assert(res.executionOrder[0] === 'root', 'Simpul awal adalah root');
  assert(res.executionOrder[3] === 'join', 'Simpul akhir adalah join');
}

// ─── TEST 3: Cycle Detection (Circular Dependency) ───
{
  const cycleDag = {
    nodes: ['node_1', 'node_2', 'node_3'],
    edges: [
      { from: 'node_1', to: 'node_2' },
      { from: 'node_2', to: 'node_3' },
      { from: 'node_3', to: 'node_1' } // Siklus fatal
    ]
  };
  const res = validateAndSortDAG(cycleDag);
  assert(res.isValid === false, 'Siklus berhasil dideteksi dan digagalkan');
  assert(res.errors[0].includes('dependensi melingkar'), 'Pesan error siklus akurat');
}

// ─── TEST 4: Forward Reference Data Binding ───
{
  const modules = [
    { id: 'step_1', params: { text: 'Halo' } },
    { id: 'step_2', params: { source: '{{step_3.output}}' } }, // Ilegal: step_3 belum jalan
    { id: 'step_3', params: { query: 'test' } }
  ];
  const order = ['step_1', 'step_2', 'step_3'];
  const res = validateDataBindings(modules, order);
  assert(res.isValid === false, 'Forward-reference data binding berhasil dicegah');
}

// ─── TEST 5: Circuit Breaker Max Steps ───
{
  const cb = new CircuitBreaker({ max_steps: 3 });
  try {
    cb.checkBeforeStep('step_1');
    cb.recordStepOutcome('step_1', 'ok');
    cb.checkBeforeStep('step_2');
    cb.recordStepOutcome('step_2', 'ok');
    cb.checkBeforeStep('step_3');
    cb.recordStepOutcome('step_3', 'ok');
    cb.checkBeforeStep('step_4'); // Seharusnya trip di sini
    assert(false, 'Seharusnya gagal karena melebihi max_steps');
  } catch (err) {
    assert(err.message.includes('MAX_STEPS_EXCEEDED'), 'Circuit breaker berhasil memutus langkah berlebih');
  }
}

// ─── TEST 6: Circuit Breaker Thrashing Loop Detection ───
{
  const cb = new CircuitBreaker({ max_steps: 10 });
  try {
    cb.checkBeforeStep('tool_loop');
    cb.recordStepOutcome('tool_loop', 'Error 500 retry');
    cb.checkBeforeStep('tool_loop');
    cb.recordStepOutcome('tool_loop', 'Error 500 retry');
    cb.checkBeforeStep('tool_loop');
    cb.recordStepOutcome('tool_loop', 'Error 500 retry'); // Aksi identik ke-3
    assert(false, 'Seharusnya gagal karena thrashing loop');
  } catch (err) {
    assert(err.message.includes('THRASHING_LOOP_DETECTED'), 'Thrashing loop berulang berhasil dicegah');
  }
}

console.log(`\n=== HASIL UJI: ${passedTests} LULUS, ${failedTests} GAGAL ===`);
process.exit(failedTests > 0 ? 1 : 0);
