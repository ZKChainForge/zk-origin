
const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("LineageVerifier", function() {
    let lineageVerifier;
    let mockVerifier;
    let owner;
    let user;
    
    beforeEach(async function() {
        [owner, user] = await ethers.getSigners();
        
        // Deploy mock Groth16 verifier (ethers v6)
        const MockVerifier = await ethers.getContractFactory("MockGroth16Verifier");
        mockVerifier = await MockVerifier.deploy();
        await mockVerifier.waitForDeployment();
        
        // Get address (ethers v6)
        const mockVerifierAddress = await mockVerifier.getAddress();
        
        // Deploy LineageVerifier
        const LineageVerifier = await ethers.getContractFactory("LineageVerifier");
        lineageVerifier = await LineageVerifier.deploy(mockVerifierAddress);
        await lineageVerifier.waitForDeployment();
    });
    
    // Helper to generate random bytes32
    function randomBytes32() {
        return ethers.hexlify(ethers.randomBytes(32));
    }
    
    describe("Genesis Setup", function() {
        it("should allow admin to set genesis", async function() {
            const genesisHash = randomBytes32();
            const genesisLineage = randomBytes32();
            
            await expect(lineageVerifier.setGenesis(genesisHash, genesisLineage))
                .to.emit(lineageVerifier, "GenesisSet")
                .withArgs(genesisHash, genesisLineage);
            
            expect(await lineageVerifier.genesisInitialized()).to.equal(true);
            expect(await lineageVerifier.hasVerifiedLineage(genesisHash)).to.equal(true);
        });
        
        it("should reject duplicate genesis setup", async function() {
            const genesisHash = randomBytes32();
            const genesisLineage = randomBytes32();
            
            await lineageVerifier.setGenesis(genesisHash, genesisLineage);
            
            await expect(
                lineageVerifier.setGenesis(genesisHash, genesisLineage)
            ).to.be.revertedWithCustomError(lineageVerifier, "GenesisAlreadySet");
        });
        
        it("should reject non-admin genesis setup", async function() {
            const genesisHash = randomBytes32();
            const genesisLineage = randomBytes32();
            
            await expect(
                lineageVerifier.connect(user).setGenesis(genesisHash, genesisLineage)
            ).to.be.revertedWithCustomError(lineageVerifier, "NotAdmin");
        });
        
        it("should reject zero state hash", async function() {
            const zeroHash = ethers.ZeroHash;
            const genesisLineage = randomBytes32();
            
            await expect(
                lineageVerifier.setGenesis(zeroHash, genesisLineage)
            ).to.be.revertedWithCustomError(lineageVerifier, "ZeroStateHash");
        });
    });
    
    describe("Lineage Verification", function() {
        beforeEach(async function() {
            const genesisHash = randomBytes32();
            const genesisLineage = randomBytes32();
            await lineageVerifier.setGenesis(genesisHash, genesisLineage);
        });
        
        it("should verify valid lineage proof", async function() {
            const pA = [1n, 2n];
            const pB = [[3n, 4n], [5n, 6n]];
            const pC = [7n, 8n];
            const publicSignals = [
                BigInt(randomBytes32()),
                1n
            ];
            
            await expect(lineageVerifier.verifyLineage(pA, pB, pC, publicSignals))
                .to.emit(lineageVerifier, "LineageVerified");
            
            expect(await lineageVerifier.totalTransitions()).to.equal(1n);
        });
        
        it("should reject without genesis", async function() {
            // Deploy fresh contract without genesis
            const mockAddr = await mockVerifier.getAddress();
            const LineageVerifier = await ethers.getContractFactory("LineageVerifier");
            const freshVerifier = await LineageVerifier.deploy(mockAddr);
            await freshVerifier.waitForDeployment();
            
            const pA = [1n, 2n];
            const pB = [[3n, 4n], [5n, 6n]];
            const pC = [7n, 8n];
            const publicSignals = [1n, 1n];
            
            await expect(
                freshVerifier.verifyLineage(pA, pB, pC, publicSignals)
            ).to.be.revertedWithCustomError(freshVerifier, "GenesisNotSet");
        });
        
        it("should reject invalid proof", async function() {
            // Set mock to return false
            await mockVerifier.setVerifyResult(false);
            
            const pA = [1n, 2n];
            const pB = [[3n, 4n], [5n, 6n]];
            const pC = [7n, 8n];
            const publicSignals = [1n, 1n];
            
            await expect(
                lineageVerifier.verifyLineage(pA, pB, pC, publicSignals)
            ).to.be.revertedWithCustomError(lineageVerifier, "InvalidProof");
        });
    });
    
    describe("View Functions", function() {
        it("should return correct state info", async function() {
            const genesisHash = randomBytes32();
            const genesisLineage = randomBytes32();
            await lineageVerifier.setGenesis(genesisHash, genesisLineage);
            
            const [lineage, depth, verified] = await lineageVerifier.getStateInfo(genesisHash);
            
            expect(lineage).to.equal(genesisLineage);
            expect(depth).to.equal(0n);
            expect(verified).to.equal(true);
        });
        
        it("should return false for unverified states", async function() {
            const randomHash = randomBytes32();
            expect(await lineageVerifier.hasVerifiedLineage(randomHash)).to.equal(false);
        });
        
        it("should return correct lineage for verified state", async function() {
            const genesisHash = randomBytes32();
            const genesisLineage = randomBytes32();
            await lineageVerifier.setGenesis(genesisHash, genesisLineage);
            
            expect(await lineageVerifier.getLineage(genesisHash)).to.equal(genesisLineage);
        });
        
        it("should return correct depth for verified state", async function() {
            const genesisHash = randomBytes32();
            const genesisLineage = randomBytes32();
            await lineageVerifier.setGenesis(genesisHash, genesisLineage);
            
            expect(await lineageVerifier.getDepth(genesisHash)).to.equal(0n);
        });
    });
    
    describe("Admin Functions", function() {
        it("should allow admin transfer", async function() {
            await lineageVerifier.transferAdmin(user.address);
            expect(await lineageVerifier.admin()).to.equal(user.address);
        });
        
        it("should reject non-admin transfer", async function() {
            await expect(
                lineageVerifier.connect(user).transferAdmin(user.address)
            ).to.be.revertedWithCustomError(lineageVerifier, "NotAdmin");
        });
        
        it("should allow new admin to set genesis after transfer", async function() {
            await lineageVerifier.transferAdmin(user.address);
            
            const genesisHash = randomBytes32();
            const genesisLineage = randomBytes32();
            
            await expect(
                lineageVerifier.connect(user).setGenesis(genesisHash, genesisLineage)
            ).to.emit(lineageVerifier, "GenesisSet");
        });
    });
    
    describe("Multiple Transitions", function() {
        beforeEach(async function() {
            const genesisHash = randomBytes32();
            const genesisLineage = randomBytes32();
            await lineageVerifier.setGenesis(genesisHash, genesisLineage);
        });
        
        it("should track multiple transitions", async function() {
            const pA = [1n, 2n];
            const pB = [[3n, 4n], [5n, 6n]];
            const pC = [7n, 8n];
            
            // First transition
            const signals1 = [BigInt(randomBytes32()), 1n];
            await lineageVerifier.verifyLineage(pA, pB, pC, signals1);
            
            // Second transition
            const signals2 = [BigInt(randomBytes32()), 2n];
            await lineageVerifier.verifyLineage(pA, pB, pC, signals2);
            
            // Third transition
            const signals3 = [BigInt(randomBytes32()), 3n];
            await lineageVerifier.verifyLineage(pA, pB, pC, signals3);
            
            expect(await lineageVerifier.totalTransitions()).to.equal(3n);
        });
    });
});
