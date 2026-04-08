
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("ZK-ORIGIN System", function () {
    let lineageVerifier;
    let groth16Verifier;
    let epochManager;
    let rateLimiter;
    let authVerifier;
    
    let owner;
    let user1;
    let user2;
    
    const GENESIS_COMMITMENT = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("genesis"));
    const POLICY_ROOT = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("policy"));
    const GENESIS_STATE_HASH = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("genesis_state"));
    const NEW_STATE_HASH = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("new_state"));
    
    beforeEach(async function () {
        [owner, user1, user2] = await ethers.getSigners();
        
        // Deploy Groth16Verifier
        const Groth16VerifierFactory = await ethers.getContractFactory("Groth16Verifier");
        groth16Verifier = await Groth16VerifierFactory.deploy();
        await groth16Verifier.deployed();
        
        // Deploy EpochManager
        const EpochManagerFactory = await ethers.getContractFactory("EpochManager");
        epochManager = await EpochManagerFactory.deploy();
        await epochManager.deployed();
        
        // Deploy RateLimiter
        const RateLimiterFactory = await ethers.getContractFactory("RateLimiter");
        rateLimiter = await RateLimiterFactory.deploy();
        await rateLimiter.deployed();
        
        // Deploy AuthorizationVerifier
        const AuthVerifierFactory = await ethers.getContractFactory("AuthorizationVerifier");
        authVerifier = await AuthVerifierFactory.deploy();
        await authVerifier.deployed();
        
        // Deploy LineageVerifier
        const LineageVerifierFactory = await ethers.getContractFactory("LineageVerifier");
        lineageVerifier = await LineageVerifierFactory.deploy(
            groth16Verifier.address,
            epochManager.address,
            rateLimiter.address,
            authVerifier.address,
            GENESIS_COMMITMENT,
            POLICY_ROOT,
            false // no duplicates
        );
        await lineageVerifier.deployed();
    });
    
    describe("Genesis Initialization", function () {
        it("Should set genesis state", async function () {
            await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_COMMITMENT);
            
            const hasGenesis = await lineageVerifier.hasVerifiedLineage(GENESIS_STATE_HASH);
            expect(hasGenesis).to.equal(true);
            
            const lineage = await lineageVerifier.getLineage(GENESIS_STATE_HASH);
            expect(lineage).to.equal(GENESIS_COMMITMENT);
            
            const depth = await lineageVerifier.getDepth(GENESIS_STATE_HASH);
            expect(depth.toString()).to.equal("0");
        });
        
        it("Should not allow duplicate genesis", async function () {
            await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_COMMITMENT);
            
            // Expect revert (any kind)
            await expect(
                lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_COMMITMENT)
            ).to.be.reverted;
        });
        
        it("Should not allow zero state hash", async function () {
            // Expect revert
            await expect(
                lineageVerifier.setGenesis(ethers.constants.HashZero, GENESIS_COMMITMENT)
            ).to.be.reverted;
        });
    });
    
    describe("Rate Limiting", function () {
        beforeEach(async function () {
            await rateLimiter.transferAdmin(lineageVerifier.address);
        });
        
        it("Should track rate limits correctly", async function () {
            const EPOCH = 0;
            const ORIGIN_USER = 1;
            
            // Check canProceed
            const canProceed = await rateLimiter.canProceed(EPOCH, ORIGIN_USER);
            expect(canProceed).to.equal(true);
            
            // Check counter is 0
            const counter = await rateLimiter.getCounter(EPOCH, ORIGIN_USER);
            expect(counter.toString()).to.equal("0");
        });
        
        it("Should respect rate limits", async function () {
            const EPOCH = 0;
            const ORIGIN_ADMIN = 2;
            
            // Set rate limit to 3
            await rateLimiter.updateRateLimit(ORIGIN_ADMIN, 3);
            
            // Increment 3 times (should succeed)
            await rateLimiter.incrementCounter(EPOCH, ORIGIN_ADMIN);
            await rateLimiter.incrementCounter(EPOCH, ORIGIN_ADMIN);
            await rateLimiter.incrementCounter(EPOCH, ORIGIN_ADMIN);
            
            // Check counter is 3
            const counter = await rateLimiter.getCounter(EPOCH, ORIGIN_ADMIN);
            expect(counter.toString()).to.equal("3");
            
            // Check canProceed is false
            const canProceed = await rateLimiter.canProceed(EPOCH, ORIGIN_ADMIN);
            expect(canProceed).to.equal(false);
        });
    });
    
    describe("Epoch Management", function () {
        it("Should track epoch correctly", async function () {
            const epoch = await epochManager.getCurrentEpoch();
            expect(typeof epoch).to.equal("object"); // BigNumber
        });
        
        it("Should update epoch", async function () {
            const epoch1 = await epochManager.getCurrentEpoch();
            await epochManager.updateEpoch();
            const epoch2 = await epochManager.getCurrentEpoch();
            
            // Epochs should be equal (no time passed)
            expect(epoch1.toString()).to.equal(epoch2.toString());
        });
    });
    
    describe("Origin Policy Enforcement", function () {
        beforeEach(async function () {
            await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_COMMITMENT);
        });
        
        it("Should allow Genesis to User transition", async function () {
            const allowed = await lineageVerifier.isTransitionAllowed(0, 1); // Genesis to User
            expect(allowed).to.equal(true);
        });
        
        it("Should allow Admin to Admin transition", async function () {
            const allowed = await lineageVerifier.isTransitionAllowed(2, 2); // Admin to Admin
            expect(allowed).to.equal(true);
        });
        
        it("Should not allow User to Admin transition", async function () {
            const allowed = await lineageVerifier.isTransitionAllowed(1, 2); // User to Admin
            expect(allowed).to.equal(false);
        });
        
        it("Should allow Governance to all", async function () {
            for (let i = 0; i < 7; i++) {
                const allowed = await lineageVerifier.isTransitionAllowed(4, i); // Governance to i
                expect(allowed).to.equal(true);
            }
        });
    });
    
    describe("Authorization Verification", function () {
        it("Should verify user signature", async function () {
            const messageHash = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("test"));
            const signature = await user1.signMessage(ethers.utils.arrayify(messageHash));
            
            const valid = await authVerifier.verifyUserSignature(
                messageHash,
                signature,
                user1.address
            );
            expect(valid).to.equal(true);
        });
        
        it("Should reject invalid signature", async function () {
            const messageHash = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("test"));
            const signature = await user1.signMessage(ethers.utils.arrayify(messageHash));
            
            const valid = await authVerifier.verifyUserSignature(
                messageHash,
                signature,
                user2.address // Wrong signer
            );
            expect(valid).to.equal(false);
        });
    });
    
    describe("System Statistics", function () {
        beforeEach(async function () {
            await lineageVerifier.setGenesis(GENESIS_STATE_HASH, GENESIS_COMMITMENT);
        });
        
        it("Should return correct statistics", async function () {
            const stats = await lineageVerifier.getStats();
            
            expect(stats.transitions.toString()).to.equal("0"); // No transitions yet
            expect(stats.maxDepth.toString()).to.equal("0");
            expect(stats.initialized).to.equal(true);
            expect(stats.isPaused).to.equal(false);
            expect(typeof stats.currentEpoch).to.equal("object"); // BigNumber
        });
    });
});
