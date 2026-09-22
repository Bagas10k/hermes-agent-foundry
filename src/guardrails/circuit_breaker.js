/**
 * Hermes Agent Foundry :: Circuit Breaker & Safety Guard
 * Mencegah runaway loops, token bleeding, dan kelelahan komputasi.
 */

class CircuitBreaker {
  /**
   * @param {Object} constraints
   * @param {number} [constraints.max_steps=10]
   * @param {number} [constraints.timeout_seconds=180]
   * @param {number} [constraints.token_budget_per_run=15000]
   */
  constructor(constraints = {}) {
    this.maxSteps = constraints.max_steps || 10;
    this.timeoutMs = (constraints.timeout_seconds || 180) * 1000;
    this.tokenBudget = constraints.token_budget_per_run || 15000;

    this.currentStep = 0;
    this.tokensUsed = 0;
    this.startTime = Date.now();
    this.actionHistory = [];
    this.isTripped = false;
    this.tripReason = null;
  }

  /**
   * Evaluasi prakondisi sebelum simpul dieksekusi
   */
  checkBeforeStep(nodeId) {
    if (this.isTripped) {
      throw new Error(`Circuit Breaker aktif [${this.tripReason}]. Eksekusi dibatalkan.`);
    }

    const elapsed = Date.now() - this.startTime;
    if (elapsed > this.timeoutMs) {
      this.trip('TIMEOUT_EXCEEDED', `Batas waktu eksekusi ${this.timeoutMs / 1000}s terlampaui`);
    }

    this.currentStep += 1;
    if (this.currentStep > this.maxSteps) {
      this.trip('MAX_STEPS_EXCEEDED', `Jumlah langkah eksekusi melebihi batas maksimum (${this.maxSteps} langkah)`);
    }

    if (this.tokensUsed >= this.tokenBudget) {
      this.trip('TOKEN_BUDGET_EXCEEDED', `Plafon konsumsi token (${this.tokenBudget}) telah habis`);
    }
  }

  /**
   * Catat penggunaan token dan periksa deteksi pengulangan aksi (thrashing loop)
   */
  recordStepOutcome(nodeId, outputData, tokenUsage = 0) {
    this.tokensUsed += tokenUsage;

    // Hitung hash sederhana atau stringified fingerprint untuk deteksi thrashing
    const serialized = typeof outputData === 'string' ? outputData : JSON.stringify(outputData || '');
    const fingerprint = `${nodeId}:${serialized.slice(0, 100)}`;
    this.actionHistory.push(fingerprint);

    // Cek apakah 3 aksi terakhir identik
    if (this.actionHistory.length >= 3) {
      const len = this.actionHistory.length;
      if (this.actionHistory[len - 1] === this.actionHistory[len - 2] &&
          this.actionHistory[len - 2] === this.actionHistory[len - 3]) {
        this.trip('THRASHING_LOOP_DETECTED', `Mendeteksi perulangan aksi identik pada modul '${nodeId}' sebanyak 3 kali berturut-turut`);
      }
    }

    if (this.tokensUsed >= this.tokenBudget) {
      this.trip('TOKEN_BUDGET_EXCEEDED', `Plafon konsumsi token (${this.tokenBudget}) telah habis`);
    }
  }

  trip(reason, message) {
    this.isTripped = true;
    this.tripReason = reason;
    const err = new Error(`[CIRCUIT_BREAKER_TRIPPED] ${reason}: ${message}`);
    err.code = reason;
    throw err;
  }

  getMetrics() {
    return {
      elapsedMs: Date.now() - this.startTime,
      currentStep: this.currentStep,
      tokensUsed: this.tokensUsed,
      isTripped: this.isTripped,
      tripReason: this.tripReason
    };
  }
}

module.exports = {
  CircuitBreaker
};
