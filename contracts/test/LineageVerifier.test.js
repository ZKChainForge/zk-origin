const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("LineageVerifier", function() {
    let lineageVerifier;
    let mockVerifier;
    let owner;
    let user;
    
    beforeEach(async function() {
        [owner, user] = await ethers.getSigners();
        
        // Deploy a mock Groth16 verifier that always returns true
        const MockVerifier = await ethers.getContractFactory("MockGroth16Verifier");
        mockVerifier = await MockVerifier.deploy();
        
        // Deploy LineageVerifier
        const LineageVerifier = await ethers.getContractFactory("LineageVerifier");
        lineageVerifier = await LineageVerifier.deploy(await mockVerifier.getAddress());
    });
    
    describe("Genesis Setup", function() {
        it("should allow admin to set genesis", async function() {
            const genesisHash = ethers.randomBytes(32);
            const genesisLineage = ethers.randomBytes(32);
            
            await expect(lineageVerifier.setGenesis(genesisHash, genesisLineage))
                .to.emit(lineageVerifier, "GenesisSet")
                .withArgs(genesisHash, genesisLineage);
            
            expect(await lineageVerifier.genesisInitialized()).to.be.true;
            expect(await lineageVerifier.hasVerifiedLineage(genesisHash)).to.be.true;
        });
        
        it("should reject duplicate genesis setup", async function() {
            const genesisHash = ethers.randomBytes(32);
            const genesisLineage = ethers.randomBytes(32);
            
            await lineageVerifier.setGenesis(genesisHash, genesisLineage);
            
            await expect(lineageVerifier.setGenesis(genesisHash, genesisLineage))
                .to.be.revertedWithCustomError(lineageVerifier, "GenesisAlreadySet");
        });
        
        it("should reject non-admin genesis setup", async function() {
            const genesisHash = ethers.randomBytes(32);
            const genesisLineage = ethers.randomBytes(32);
            
            await expect(lineageVerifier.connect(user).setGenesis(genesisHash, genesisLineage))
                .to.be.revertedWithCustomError(lineageVerifier, "NotAdmin");
        });
        
        it("should reject zero state hash", async function() {
            const zeroHash = ethers.ZeroHash;
            const genesisLineage = ethers.randomBytes(32);
            
            await expect(lineageVerifier.setGenesis(zeroHash, genesisLineage))
                .to.be.revertedWithCustomError(lineageVerifier, "ZeroStateHash");
        });
    });
    
    describe("Lineage Verification", function() {
        beforeEach(async function() {
            const genesisHash = ethers.randomBytes(32);
            const genesisLineage = ethers.randomBytes(32);
            await lineageVerifier.setGenesis(genesisHash, genesisLineage);
        });
        
        it("should verify valid lineage proof", async function() {
            const pA = [1n, 2n];
            const pB = [[3n, 4n], [5n, 6n]];
            const pC = [7n, 8n];
            const publicSignals = [
                BigInt(ethers.hexlify(ethers.randomBytes(32))), // new lineage
                1n // depth
            ];
            
            await expect(lineageVerifier.verifyLineage(pA, pB, pC, publicSignals))
                .to.emit(lineageVerifier, "LineageVerified");
            
            expect(await lineageVerifier.totalTransitions()).to.equal(1);
        });
        
        it("should reject without genesis", async function() {
            // Deploy fresh contract without genesis
            const freshVerifier = await (await ethers.getContractFactory("LineageVerifier"))
                .deploy(await mockVerifier.getAddress());
            
            const pA = [1n, 2n];
            const pB = [[3n, 4n], [5n, 6n]];
            const pC = [7n, 8n];
            const publicSignals = [1n, 1n];
            
            await expect(freshVerifier.verifyLineage(pA, pB, pC, publicSignals))
                .to.be.revertedWithCustomError(freshVerifier, "GenesisNotSet");
        });
    });
    
    describe("View Functions", function() {
        it("should return correct state info", async function() {
            const genesisHash = ethers.randomBytes(32);
            const genesisLineage = ethers.randomBytes(32);
            await lineageVerifier.setGenesis(genesisHash, genesisLineage);
            
            const [lineage, depth, verified] = await lineageVerifier.getStateInfo(genesisHash);
            
            expect(lineage).to.equal(ethers.hexlify(genesisLineage));
            expect(depth).to.equal(0);
            expect(verified).to.be.true;
        });
        
        it("should return false for unverified states", async function() {
            const randomHash = ethers.randomBytes(32);
            expect(await lineageVerifier.hasVerifiedLineage(randomHash)).to.be.false;
        });
    });
    
    describe("Admin Functions", function() {
        it("should allow admin transfer", async function() {
            await lineageVerifier.transferAdmin(user.address);
            expect(await lineageVerifier.admin()).to.equal(user.address);
        });
        
        it("should reject non-admin transfer", async function() {
            await expect(lineageVerifier.connect(user).transferAdmin(user.address))
                .to.be.revertedWithCustomError(lineageVerifier, "NotAdmin");
        });
    });
});

