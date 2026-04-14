/**
 * Real-time Monitoring & Alerting
 * 
 * Monitors contract state and emits alerts for anomalies
 */

const { ethers } = require("hardhat");
const EventEmitter = require("events");
const fs = require("fs");

class ContractMonitor extends EventEmitter {
    constructor(contractAddress, pollInterval = 5000) {
        super();
        this.contractAddress = contractAddress;
        this.pollInterval = pollInterval;
        this.previousState = null;
        this.alerts = [];
        this.metrics = {
            totalProofsVerified: 0,
            totalGasUsed: 0,
            totalFeePaid: 0,
            avgProofTime: 0,
            maxDepth: 0,
        };
    }
    
    /**
     * Start monitoring
     */
    async start() {
        console.log("\n" + "═".repeat(60));
        console.log("📊 CONTRACT MONITORING STARTED");
        console.log("═".repeat(60));
        
        const lineageVerifier = await ethers.getContractAt(
            "LineageVerifier",
            this.contractAddress
        );
        
        // Initial state
        let previousStats = await lineageVerifier.getStats();
        
        // Poll contract
        setInterval(async () => {
            try {
                const currentStats = await lineageVerifier.getStats();
                
                // Check for new transitions
                if (currentStats.transitions > previousStats.transitions) {
                    this.onNewTransition(currentStats, previousStats);
                }
                
                // Check for epoch change
                if (currentStats.currentEpoch > previousStats.currentEpoch) {
                    this.onEpochChange(currentStats);
                }
                
                // Check for pause
                if (currentStats.isPaused !== previousStats.isPaused) {
                    this.onPauseChange(currentStats);
                }
                
                previousStats = currentStats;
                
                // Emit metrics
                this.emit("metrics", this.metrics);
                
            } catch (error) {
                this.createAlert("MONITOR_ERROR", error.message);
            }
        }, this.pollInterval);
    }
    
    /**
     * Handle new transition
     */
    onNewTransition(current, previous) {
        const newTransitions = current.transitions - previous.transitions;
        
        console.log(`\n✅ NEW TRANSITIONS: +${newTransitions}`);
        console.log(`   Total: ${current.transitions}`);
        console.log(`   Depth: ${current.maxDepth}`);
        
        // Update metrics
        this.metrics.totalProofsVerified = current.transitions;
        this.metrics.maxDepth = current.maxDepth;
        
        // Check for rate limit anomalies
        if (newTransitions > 100) {
            this.createAlert("HIGH_THROUGHPUT", `${newTransitions} proofs in one interval`);
        }
        
        this.emit("newTransition", {
            count: newTransitions,
            total: current.transitions,
            depth: current.maxDepth,
        });
    }
    
    /**
     * Handle epoch change
     */
    onEpochChange(stats) {
        console.log(`\n🔄 EPOCH CHANGE DETECTED`);
        console.log(`   Epoch: ${stats.currentEpoch}`);
        console.log(`   Counters reset: yes`);
        
        this.createAlert("EPOCH_CHANGE", `Epoch ${stats.currentEpoch} started`);
        
        this.emit("epochChange", {
            epochId: stats.currentEpoch,
        });
    }
    
    /**
     * Handle pause state change
     */
    onPauseChange(stats) {
        console.log(`\n⚠️  PAUSE STATE CHANGED: ${stats.isPaused ? "PAUSED" : "RESUMED"}`);
        
        if (stats.isPaused) {
            this.createAlert("CONTRACT_PAUSED", "Verification paused by admin");
        } else {
            this.createAlert("CONTRACT_RESUMED", "Verification resumed");
        }
        
        this.emit("pauseChange", {
            paused: stats.isPaused,
        });
    }
    
    /**
     * Create alert
     */
    createAlert(type, message) {
        const alert = {
            type,
            message,
            timestamp: new Date().toISOString(),
        };
        
        this.alerts.push(alert);
        
        // Keep last 100 alerts
        if (this.alerts.length > 100) {
            this.alerts.shift();
        }
        
        console.log(`\n⚠️  ALERT: [${type}] ${message}`);
        this.emit("alert", alert);
    }
    
    /**
     * Watch for events
     */
    async watchEvents() {
        const lineageVerifier = await ethers.getContractAt(
            "LineageVerifier",
            this.contractAddress
        );
        
        // Listen for LineageVerified events
        lineageVerifier.on("LineageVerified", (prev, next, lineage, depth, origin, epoch, creator) => {
            console.log(`\n📋 EVENT: LineageVerified`);
            console.log(`   Prev: ${prev.slice(0, 10)}...`);
            console.log(`   Next: ${next.slice(0, 10)}...`);
            console.log(`   Depth: ${depth}`);
            console.log(`   Origin: ${origin}`);
            
            this.emit("event", {
                type: "LineageVerified",
                prev,
                next,
                depth: depth.toNumber(),
            });
        });
    }
    
    /**
     * Get statistics
     */
    getStats() {
        return {
            metrics: this.metrics,
            alerts: this.alerts,
            alertCount: this.alerts.length,
        };
    }
    
    /**
     * Save statistics
     */
    saveStats(filename = "monitoring_stats.json") {
        fs.writeFileSync(
            filename,
            JSON.stringify(this.getStats(), null, 2)
        );
        console.log(`\n💾 Stats saved to ${filename}`);
    }
}

/**
 * Main monitoring loop
 */
async function main() {
    const contractAddress = process.env.LINEAGE_VERIFIER || 
        "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9";
    
    const monitor = new ContractMonitor(contractAddress, 5000);
    
    // Handle signals
    process.on("SIGINT", () => {
        console.log("\n\n🛑 Monitoring stopped");
        monitor.saveStats();
        process.exit(0);
    });
    
    // Listen to events
    monitor.on("newTransition", (data) => {
        console.log(`[EVENT] New transitions: ${data.count}`);
    });
    
    monitor.on("alert", (alert) => {
        console.log(`[ALERT] ${alert.type}: ${alert.message}`);
    });
    
    monitor.on("metrics", (metrics) => {
        console.log(`[METRICS] Proofs: ${metrics.totalProofsVerified}, MaxDepth: ${metrics.maxDepth}`);
    });
    
    // Start monitoring
    await monitor.start();
    await monitor.watchEvents();
    
    // Keep running
    await new Promise(() => {});
}

main().catch(console.error);