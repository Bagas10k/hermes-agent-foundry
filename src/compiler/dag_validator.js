/**
 * Hermes Agent Foundry :: DAG Validator
 * Memverifikasi integritas graf eksekusi agen:
 * 1. Deteksi Siklus (Kahn's Algorithm - Topological Sort)
 * 2. Validasi Integritas Edge (Tidak ada orphan atau missing nodes)
 * 3. Validasi Upstream Binding (Tidak ada forward-reference data injection)
 */

class DAGValidationError extends Error {
  constructor(message, details = {}) {
    super(message);
    this.name = 'DAGValidationError';
    this.details = details;
  }
}

/**
 * Memvalidasi dan mengurutkan simpul secara topologis
 * @param {Object} pipelineDag - { nodes: string[], edges: Array<{from: string, to: string}> }
 * @returns {{ isValid: boolean, executionOrder: string[], errors: string[] }}
 */
function validateAndSortDAG(pipelineDag) {
  const errors = [];
  const nodes = pipelineDag.nodes || [];
  const edges = pipelineDag.edges || [];

  if (nodes.length === 0) {
    return {
      isValid: false,
      executionOrder: [],
      errors: ['Pipeline DAG tidak memiliki simpul modul (nodes kosong)']
    };
  }

  // 1. Cek keberadaan node pada edges
  const nodeSet = new Set(nodes);
  edges.forEach((edge, idx) => {
    if (!nodeSet.has(edge.from)) {
      errors.push(`Edge index ${idx}: simpul asal '${edge.from}' tidak terdaftar di nodes`);
    }
    if (!nodeSet.has(edge.to)) {
      errors.push(`Edge index ${idx}: simpul tujuan '${edge.to}' tidak terdaftar di nodes`);
    }
    if (edge.from === edge.to) {
      errors.push(`Edge index ${idx}: mendeteksi self-loop pada simpul '${edge.from}'`);
    }
  });

  if (errors.length > 0) {
    return { isValid: false, executionOrder: [], errors };
  }

  // 2. Kahn's Algorithm untuk Topological Sort & Cycle Detection
  const inDegree = new Map();
  const adjacency = new Map();

  nodes.forEach(node => {
    inDegree.set(node, 0);
    adjacency.set(node, []);
  });

  edges.forEach(edge => {
    inDegree.set(edge.to, inDegree.get(edge.to) + 1);
    adjacency.get(edge.from).push(edge.to);
  });

  // Antrean simpul dengan in-degree 0 (tanpa dependensi hulu)
  const queue = [];
  nodes.forEach(node => {
    if (inDegree.get(node) === 0) {
      queue.push(node);
    }
  });

  const executionOrder = [];

  while (queue.length > 0) {
    const current = queue.shift();
    executionOrder.push(current);

    const neighbors = adjacency.get(current) || [];
    neighbors.forEach(neighbor => {
      inDegree.set(neighbor, inDegree.get(neighbor) - 1);
      if (inDegree.get(neighbor) === 0) {
        queue.push(neighbor);
      }
    });
  }

  // Jika jumlah simpul tereksekusi < total simpul, berarti terdapat siklus (Deadlock / Circular Dependency)
  if (executionOrder.length !== nodes.length) {
    const unvisited = nodes.filter(n => inDegree.get(n) > 0);
    errors.push(`Mendeteksi dependensi melingkar (cycle) pada simpul: [${unvisited.join(', ')}]`);
    return { isValid: false, executionOrder: [], errors };
  }

  return {
    isValid: true,
    executionOrder,
    errors: []
  };
}

/**
 * Validasi Binding Data: memastikan referensi {{nodeId.output}} hanya merujuk ke simpul upstream
 */
function validateDataBindings(modules, executionOrder) {
  const errors = [];
  const moduleMap = new Map(modules.map(m => [m.id, m]));
  const orderIndexMap = new Map(executionOrder.map((id, idx) => [id, idx]));

  const bindingRegex = /\{\{([a-zA-Z0-9_-]+)\.([a-zA-Z0-9_.-]+)\}\}/g;

  modules.forEach(mod => {
    const modOrder = orderIndexMap.get(mod.id);
    const modString = JSON.stringify(mod.params || {});
    let match;

    while ((match = bindingRegex.exec(modString)) !== null) {
      const referencedNodeId = match[1];

      // Abaikan variabel global bawaan seperti {{date_today}}, {{input.x}}
      if (referencedNodeId === 'input' || referencedNodeId === 'date_today' || referencedNodeId === 'env') {
        continue;
      }

      if (!orderIndexMap.has(referencedNodeId)) {
        errors.push(`Modul '${mod.id}' merujuk simpul '${referencedNodeId}' yang tidak terdaftar`);
        continue;
      }

      const refOrder = orderIndexMap.get(referencedNodeId);
      if (refOrder >= modOrder) {
        errors.push(`Pelanggaran alur data: Modul '${mod.id}' (urutan ${modOrder}) merujuk ke modul '${referencedNodeId}' (urutan ${refOrder}) yang belum dieksekusi`);
      }
    }
  });

  return {
    isValid: errors.length === 0,
    errors
  };
}

module.exports = {
  validateAndSortDAG,
  validateDataBindings,
  DAGValidationError
};
