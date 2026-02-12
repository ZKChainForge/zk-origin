module.exports = {
  projectName: "zk-origin",
  outputDir: "./build",
  circuits: [
    {
      name: "lineage_step",
      path: "./src/main/lineage_step.circom",
      publicSignals: ["prev_state_hash", "new_state_hash", "prev_lineage_commitment"]
    }
  ],
  snarkjs: {
    ptau: "pot12_final.ptau",
    groth16: true
  }
};