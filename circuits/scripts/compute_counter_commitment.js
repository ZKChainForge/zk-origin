const { buildPoseidon } = require("circomlibjs");

async function computeCounterCommitment() {
    const poseidon = await buildPoseidon();
    
    const epochId = 0;
    const prevCounters = [0, 0, 0, 0, 0, 0, 0];
    
    // Compute: Poseidon(epochId, counter[0], counter[1], ..., counter[6])
    const inputs = [epochId, ...prevCounters];
    const commitment = poseidon.F.toString(poseidon(inputs));
    
    console.log("Epoch ID:", epochId);
    console.log("Counters:", prevCounters);
    console.log("Counter Commitment:", commitment);
    
    return commitment;
}

computeCounterCommitment().catch(console.error);