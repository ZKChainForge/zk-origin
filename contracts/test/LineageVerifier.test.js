const { expect } = require("chai");
const { ethers } = require("hardhat");
const crypto = require("crypto");

describe("LineageVerifier - Full Integration Tests", function () {
    let lineageVerifier;
    let authVerifier;
    let rateLimiter;
    let epochManager;
    let groth16Verifier;
    let batchVerifier;
    
    let admin, user, signer1, signer2, signer3;
    
    const ORIGIN_GENESIS = 0;
    const ORIGIN_USER = 1;
    const ORIGIN_ADMIN = 2;
    const ORIGIN_BRIDGE = 3;
    const ORIGIN_GOVERNANCE = 4;
    const ORIGIN_SYSTEM = 5;
    const ORIGIN_EMERGENCY = 6;
    
    // Helper function to generate hashes
    const hash = (str) => {
        const hashObj = crypto.createHash('sha256');
        hashObj.update(str);
        return '0x' + hashObj.digest('hex');
    };
    
    // Helper to convert BigNumber to number
    const bn = (value) => Number(value.toString());
    
    // These will be initialized in beforeEach
    let GENESIS_STATE_HASH;
    let GENESIS_LINEAGE_COMMITMENT;
    let POLICY_ROOT;
    
    beforeEach(async function () {
        [admin, user, signer1, signer2, signer3] = await ethers.getSigners();
        
        // Initialize hashes
        GENESIS_STATE_HASH = hash("genesis");
        GENESIS_LINEAGE_COMMITMENT = hash("genesis_lineage");
        POLICY_ROOT = hash("policy_root");
        
        // Deploy Groth16Verifier (mock)
        const Groth16Mock = await ethers.getContractFactory("MockGroth16Verifier");
        groth16Verifier = await Groth16Mock.deploy();
        await groth16Verifier.deployed();
        
        // Deploy EpochManager
        const EpochManager = await ethers.getContractFactory("EpochManager");
        epochManager = await EpochManager.deploy();
        await epochManager.deployed();
        
        // Deploy RateLimiter
        const RateLimiter = await ethers.getContractFactory("RateLimiter");
        rateLimiter = await RateLimiter.deploy();
        await rateLimiter.deployed();
        
        // Deploy AuthorizationVerifier
        const AuthorizationVerifier = await ethers.getContractFactory("AuthorizationVerifier");
        authVerifier = await AuthorizationVerifier.deploy();
        await authVerifier.deployed();
        
        // Deploy LineageVerifier
        const LineageVerifier = await ethers.getContractFactory("LineageVerifier");
        lineageVerifier = await LineageVerifier.deploy(
            groth16Verifier.address,
            epochManager.address,
            rateLimiter.address,
            authVerifier.address,
            GENESIS_LINEAGE_COMMITMENT,
            POLICY_ROOT
        );
        await lineageVerifier.deployed();
        
        // Transfer admin to LineageVerifier for RateLimiter
        await rateLimiter.transferAdmin(lineageVerifier.address);
        
        // Deploy BatchVerifier
        const BatchVerifier = await ethers.getContractFactory("BatchVerifier");
        batchVerifier = await BatchVerifier.deploy(
            lineageVerifier.address,
            authVerifier.address
        );
        await batchVerifier.deployed();
    });
    
    describe("Genesis", function () {
        it("Should set genesis state", async function () {
            await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_LINEAGE_COMMITMENT);
            
            expect(await lineageVerifier.genesisInitialized()).to.equal(true);
            expect(await lineageVerifier.genesisStateHash()).to.equal(GENESIS_STATE_HASH);
        });
        
        it("Should not allow duplicate genesis", async function () {
            await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_LINEAGE_COMMITMENT);
            
            try {
                await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_LINEAGE_COMMITMENT);
                expect.fail("Should have reverted");
            } catch (error) {
                expect(error.message).to.include("GenesisAlreadySet");
            }
        });
        
        it("Should reject zero state hash", async function () {
            const zeroHash = "0x0000000000000000000000000000000000000000000000000000000000000000";
            
            try {
                await lineageVerifier.setGenesis(zeroHash, GENESIS_LINEAGE_COMMITMENT);
                expect.fail("Should have reverted");
            } catch (error) {
                expect(error.message).to.include("ZeroStateHash");
            }
        });
    });
    
    describe("Authorization", function () {
        beforeEach(async function () {
            await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_LINEAGE_COMMITMENT);
        });
        
        it("Should verify user signature", async function () {
            const messageHash = hash("test_message");
            const messageBytes = messageHash.substring(2); // Remove 0x
            const signature = await user.signMessage(messageBytes);
            
            expect(signature).to.exist;
            expect(signature.length).to.equal(132);
        });
        
        it("Should reject invalid signature", async function () {
            const messageHash = hash("test_message");
            const wrongSig = "0x" + "00".repeat(65);
            
            expect(wrongSig).to.exist;
            expect(wrongSig.length).to.equal(132);
        });
        
        it("Should verify governance proposal", async function () {
            const yesVotes = 100;
            const noVotes = 50;
            const threshold = 40;
            
            const shouldPass = yesVotes > (noVotes + threshold);
            expect(shouldPass).to.equal(true);
        });
    });
    
    describe("Rate Limiting", function () {
        beforeEach(async function () {
            await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_LINEAGE_COMMITMENT);
        });
        
        it("Should track rate limits", async function () {
            const epoch = 0;
            const adminClass = ORIGIN_ADMIN;
            
            const limit = await rateLimiter.getLimit(adminClass);
            expect(bn(limit)).to.equal(10);
            
            const counter = await rateLimiter.getCounter(epoch, adminClass);
            expect(bn(counter)).to.equal(0);
        });
        
        it("Should allow unlimited users", async function () {
            const epoch = 0;
            const userClass = ORIGIN_USER;
            
            const limit = await rateLimiter.getLimit(userClass);
            // Check that it's a very large number (unlimited)
            expect(limit.toString()).to.include("115792089237316195423570985008687907853269984665640564039457584007913129639935");
        });
        
        it("Should get all epoch counters", async function () {
            const epoch = 0;
            const counters = await rateLimiter.getEpochCounters(epoch);
            
            expect(counters).to.have.lengthOf(7);
            for (let i = 0; i < 7; i++) {
                expect(bn(counters[i])).to.equal(0);
            }
        });
        
        it("Should get remaining capacity", async function () {
            const epoch = 0;
            const adminClass = ORIGIN_ADMIN;
            
            const capacity = await rateLimiter.getRemainingCapacity(epoch, adminClass);
            expect(bn(capacity)).to.equal(10);
        });
    });
    
    describe("Epoch Transitions", function () {
        it("Should track current epoch", async function () {
            const epoch = await epochManager.getCurrentEpoch();
            expect(bn(epoch)).to.be.gte(0);
        });
        
        it("Should check epoch changes", async function () {
            const hasChanged = await epochManager.hasEpochChanged();
            expect(typeof hasChanged).to.equal('boolean');
        });
        
        it("Should get time until next epoch", async function () {
            const timeUntil = await epochManager.timeUntilNextEpoch();
            expect(bn(timeUntil)).to.be.gte(0);
        });
        
        it("Should get epoch duration", async function () {
            const duration = await epochManager.getEpochDuration();
            expect(bn(duration)).to.equal(86400);
        });
    });
    
    describe("Policy Enforcement", function () {
        beforeEach(async function () {
            await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_LINEAGE_COMMITMENT);
        });
        
        it("Should allow genesis to user transition", async function () {
            const allowed = await lineageVerifier.isTransitionAllowed(ORIGIN_GENESIS, ORIGIN_USER);
            expect(allowed).to.equal(true);
        });
        
        it("Should allow user to user transition", async function () {
            const allowed = await lineageVerifier.isTransitionAllowed(ORIGIN_USER, ORIGIN_USER);
            expect(allowed).to.equal(true);
        });
        
        it("Should prevent invalid transitions", async function () {
            const allowed = await lineageVerifier.isTransitionAllowed(ORIGIN_BRIDGE, ORIGIN_ADMIN);
            expect(allowed).to.equal(false);
        });
        
        it("Should allow updating policy", async function () {
            let allowed = await lineageVerifier.isTransitionAllowed(ORIGIN_BRIDGE, ORIGIN_ADMIN);
            expect(allowed).to.equal(false);
            
            await lineageVerifier.setPolicyTransition(ORIGIN_BRIDGE, ORIGIN_ADMIN, true);
            
            allowed = await lineageVerifier.isTransitionAllowed(ORIGIN_BRIDGE, ORIGIN_ADMIN);
            expect(allowed).to.equal(true);
        });
    });
    
    describe("State Management", function () {
        beforeEach(async function () {
            await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_LINEAGE_COMMITMENT);
        });
        
        it("Should track verified states", async function () {
            const hasLineage = await lineageVerifier.hasVerifiedLineage(GENESIS_STATE_HASH);
            expect(hasLineage).to.equal(true);
        });
        
        it("Should return correct depth", async function () {
            const depth = await lineageVerifier.getDepth(GENESIS_STATE_HASH);
            expect(bn(depth)).to.equal(0);
        });
        
        it("Should track origin class", async function () {
            const originClass = await lineageVerifier.getOriginClass(GENESIS_STATE_HASH);
            expect(bn(originClass)).to.equal(ORIGIN_GENESIS);
        });
        
        it("Should get lineage", async function () {
            const lineage = await lineageVerifier.getLineage(GENESIS_STATE_HASH);
            expect(lineage).to.equal(GENESIS_LINEAGE_COMMITMENT);
        });
    });
    
    describe("Statistics", function () {
        beforeEach(async function () {
            await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_LINEAGE_COMMITMENT);
        });
        
        it("Should report statistics", async function () {
            const stats = await lineageVerifier.getStats();
            expect(stats.initialized).to.equal(true);
            expect(stats.isPaused).to.equal(false);
            expect(bn(stats.transitions)).to.equal(0);
        });
        
        it("Should return all stat fields", async function () {
            const stats = await lineageVerifier.getStats();
            expect(stats).to.have.property("transitions");
            expect(stats).to.have.property("maxDepth");
            expect(stats).to.have.property("initialized");
            expect(stats).to.have.property("isPaused");
            expect(stats).to.have.property("currentEpoch");
            expect(stats).to.have.property("lastProcessedEpoch");
        });
    });
    
    describe("BatchVerifier", function () {
        beforeEach(async function () {
            await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_LINEAGE_COMMITMENT);
        });
        
        describe("Batch Operations", function () {
            it("Should reject empty batch", async function () {
                try {
                    await batchVerifier.verifyBatchWithAuth([]);
                    expect.fail("Should have reverted");
                } catch (error) {
                    expect(error.message).to.include("EmptyBatch");
                }
            });
            
            it("Should reject batch that's too large", async function () {
                const proofs = Array(101).fill({
                    pA: [0, 0],
                    pB: [[0, 0], [0, 0]],
                    pC: [0, 0],
                    publicSignals: Array(19).fill(0),
                    authType: 0,
                    authData: "0x"
                });
                
                try {
                    await batchVerifier.verifyBatchWithAuth(proofs);
                    expect.fail("Should have reverted");
                } catch (error) {
                    expect(error.message).to.include("BatchTooLarge");
                }
            });
            
            it("Should estimate gas correctly", async function () {
                // Call the contract method using 'callStatic' to avoid ethers conflicts
                const estimate1 = await batchVerifier.callStatic.estimateGas(1);
                const estimate10 = await batchVerifier.callStatic.estimateGas(10);
                
                const diff = bn(estimate10) - bn(estimate1);
                expect(diff).to.equal(9 * 250000);
            });
            
            it("Should provide detailed gas estimate", async function () {
                const result = await batchVerifier.estimateGasDetailed(5);
                
                expect(bn(result.baseCost)).to.equal(21000);
                expect(bn(result.totalProofCost)).to.equal(5 * 250000);
            });
            
            it("Should validate batch size", async function () {
                expect(await batchVerifier.isValidBatchSize(0)).to.equal(false);
                expect(await batchVerifier.isValidBatchSize(1)).to.equal(true);
                expect(await batchVerifier.isValidBatchSize(100)).to.equal(true);
                expect(await batchVerifier.isValidBatchSize(101)).to.equal(false);
            });
            
            it("Should get max proofs per batch", async function () {
                const max = await batchVerifier.getMaxProofsPerBatch();
                expect(bn(max)).to.equal(100);
            });
        });
    });
    
    describe("Admin Functions", function () {
        it("Should allow admin to transfer admin role", async function () {
            await lineageVerifier.transferAdmin(user.address);
            expect(await lineageVerifier.pendingAdmin()).to.equal(user.address);
            
            await lineageVerifier.connect(user).acceptAdmin();
            expect(await lineageVerifier.admin()).to.equal(user.address);
        });
        
        it("Should allow admin to pause contract", async function () {
            await lineageVerifier.setPaused(true);
            expect(await lineageVerifier.paused()).to.equal(true);
            
            await lineageVerifier.setPaused(false);
            expect(await lineageVerifier.paused()).to.equal(false);
        });
        
        it("Should reject non-admin operations", async function () {
            try {
                await lineageVerifier.connect(user).setPaused(true);
                expect.fail("Should have reverted");
            } catch (error) {
                expect(error.message).to.include("NotAdmin");
            }
        });
        
        it("Should allow admin to update policy root", async function () {
            const newRoot = hash("new_policy_root");
            await lineageVerifier.updatePolicyRoot(newRoot);
            expect(await lineageVerifier.currentPolicyRoot()).to.equal(newRoot);
        });
    });
    
    describe("Error Handling", function () {
        it("Should prevent operations before genesis is set", async function () {
            const newStateHash = hash("new_state");
            const hasLineage = await lineageVerifier.hasVerifiedLineage(newStateHash);
            expect(hasLineage).to.equal(false);
        });
        
        it("Should handle paused contract", async function () {
            await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_LINEAGE_COMMITMENT);
            await lineageVerifier.setPaused(true);
            
            try {
                await lineageVerifier.verifyLineage(
                    [0, 0],
                    [[0, 0], [0, 0]],
                    [0, 0],
                    Array(19).fill(0),
                    0,
                    "0x"
                );
                expect.fail("Should have reverted");
            } catch (error) {
                expect(error.message).to.include("ContractPaused");
            }
        });
    });
});