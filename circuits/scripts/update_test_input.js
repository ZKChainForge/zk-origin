const { buildPoseidon } = require("circomlibjs");
const fs = require("fs");

async function generateValidTestInput() {
    const poseidon = await buildPoseidon();
    
    // Load policy tree
    const policyTree = JSON.parse(fs.readFileSync("policy_tree.json", "utf8"));
    
    // Circuit inputs
    const epochId = 0;
    const prevCounters = [0, 0, 0, 0, 0, 0, 0];
    
    // Compute prevCounterCommitment = Poseidon(epochId, counters[0..6])
    const counterInputs = [epochId, ...prevCounters];
    const prevCounterCommitment = poseidon.F.toString(poseidon(counterInputs));
    
    console.log(" Computed prevCounterCommitment:", prevCounterCommitment);
    
    // Compute prevLineageCommitment (for genesis)
    // For genesis at depth 0, this should be Poseidon(genesisStateHash, ORIGIN_GENESIS, 0)
    const genesisStateHash = 0;
    const genesisOriginClass = 0;
    const genesisDepth = 0;
    
    const prevLineageCommitment = poseidon.F.toString(
        poseidon([genesisStateHash, genesisOriginClass, genesisDepth])
    );
    
    console.log(" Computed prevLineageCommitment:", prevLineageCommitment);
    
    const testInput = {
        "prevStateHash": "0",
        "newStateHash": "1",
        "epochId": "0",
        "prevOriginClass": "0",
        "newOriginClass": "1",
        "prevLineageCommitment": prevLineageCommitment,
        "prevCounterCommitment": prevCounterCommitment,
        "policyRoot": policyTree.root,
        "expectedGenesisHash": "0",
        "prevEpochId": "0",
        "prevDepth": "0",
        "nonce": "1",
        "prevNonce": "0",
        "timestamp": "1000",
        "prevTimestamp": "0",
        "policyProof": policyTree.proof.proof,
        "policyIndices": policyTree.proof.indices,
        "prevCounters": prevCounters.map(String),
        "rateLimits": ["1", "4294967295", "10", "100", "5", "1000", "1"],
        "authorizationValid": "1"
    };
    
    fs.writeFileSync("test/inputs/main_input.json", JSON.stringify(testInput, null, 2));
    console.log(JSON.stringify(testInput, null, 2));
}

generateValidTestInput().catch(console.error);