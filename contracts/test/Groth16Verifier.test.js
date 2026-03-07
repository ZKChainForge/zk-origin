const { expect } = require("chai");
const { ethers } = require("hardhat");
const { loadFixture } = require("@nomicfoundation/hardhat-toolbox/network-helpers");

describe("Groth16Verifier", function () {
  async function deployVerifierFixture() {
    const [owner] = await ethers.getSigners();

    const Groth16Verifier = await ethers.getContractFactory("Groth16Verifier");
    const verifier = await Groth16Verifier.deploy();
    await verifier.waitForDeployment();

    return { verifier, owner };
  }

  describe("Deployment", function () {
    it("Should deploy successfully", async function () {
      const { verifier } = await loadFixture(deployVerifierFixture);
      expect(await verifier.getAddress()).to.be.properAddress;
    });

    it("Should have correct number of public inputs", async function () {
      const { verifier } = await loadFixture(deployVerifierFixture);
      expect(await verifier.getNumPublicInputs()).to.equal(5);
    });

    it("Should measure deployment gas", async function () {
      const Groth16Verifier = await ethers.getContractFactory("Groth16Verifier");
      const tx = await Groth16Verifier.deploy();
      const receipt = await tx.deploymentTransaction().wait();
      
      console.log(`    Groth16Verifier deployment gas: ${receipt.gasUsed.toString()}`);
      expect(receipt.gasUsed).to.be.lessThan(3000000);
    });
  });

  describe("Input Validation", function () {
    it("Should reject wrong number of inputs", async function () {
      const { verifier } = await loadFixture(deployVerifierFixture);
      
      const proof = {
        a: [1n, 2n],
        b: [[1n, 2n], [3n, 4n]],
        c: [1n, 2n]
      };

      // Too few inputs
      await expect(
        verifier.verifyProof(proof.a, proof.b, proof.c, [1n, 2n])
      ).to.be.revertedWithCustomError(verifier, "InvalidInputLength");

      // Too many inputs
      await expect(
        verifier.verifyProof(proof.a, proof.b, proof.c, [1n, 2n, 3n, 4n, 5n, 6n])
      ).to.be.revertedWithCustomError(verifier, "InvalidInputLength");
    });
  });

  describe("Verification", function () {
    it("Should reject invalid proof (placeholder VK)", async function () {
      const { verifier } = await loadFixture(deployVerifierFixture);
      
      // This proof will fail because we're using placeholder VK values
      const proof = {
        a: [1n, 2n],
        b: [[1n, 2n], [3n, 4n]],
        c: [1n, 2n]
      };
      const inputs = [1n, 2n, 3n, 4n, 5n];

      // Should revert due to pairing failure or return false
      // The exact behavior depends on the precompile
      try {
        const result = await verifier.verifyProof(proof.a, proof.b, proof.c, inputs);
        expect(result).to.be.false;
      } catch (error) {
        // Pairing precompile may revert with invalid points
        expect(error.message).to.include("revert");
      }
    });
  });

  describe("Gas Estimation", function () {
    it("Should estimate verification gas", async function () {
      const { verifier } = await loadFixture(deployVerifierFixture);
      
      const proof = {
        a: [1n, 2n],
        b: [[1n, 2n], [3n, 4n]],
        c: [1n, 2n]
      };
      const inputs = [1n, 2n, 3n, 4n, 5n];

      try {
        const gasEstimate = await verifier.verifyProof.estimateGas(
          proof.a, proof.b, proof.c, inputs
        );
        console.log(`    Estimated verification gas: ${gasEstimate.toString()}`);
      } catch (error) {
        // Estimation may fail with invalid points
        console.log(`    Gas estimation failed (expected with placeholder VK)`);
      }
    });
  });
});