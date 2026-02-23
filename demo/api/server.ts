import express, { Request, Response } from 'express';
import { LineageIndexer } from '../indexer/indexer';
import { LineageProver } from '../prover';

const app = express();
app.use(express.json());

let indexer: LineageIndexer | null = null;
let prover: LineageProver | null = null;

/**
 * Helper to safely extract error message
 */
function getErrorMessage(error: unknown): string {
    if (error instanceof Error) return error.message;
    return String(error);
}

/**
 * Health check
 */
app.get('/health', (_req: Request, res: Response) => {
    res.json({ status: 'ok', timestamp: Date.now() });
});

/**
 * Get lineage for a state
 */
app.get('/api/v1/lineage/:stateHash', (req: Request, res: Response) => {
    try {
        if (!indexer) {
            return res.status(500).json({ error: 'Indexer not initialized' });
        }

        const stateHash: string = req.params.stateHash;

      const history = indexer.getLineageHistory(stateHash);

        res.json({
            stateHash: req.params.stateHash,
            depth: history.length,
            history: history.map(e => ({
                stateHash: e.newStateHash,
                lineageCommitment: e.lineageCommitment,
                depth: e.depth,
                blockNumber: e.blockNumber,
                transactionHash: e.transactionHash
            }))
        });
    } catch (error) {
        res.status(500).json({
            error: 'Failed to fetch lineage',
            details: getErrorMessage(error)
        });
    }
});

/**
 * Generate proof for transition
 */
app.post('/api/v1/prove', async (req: Request, res: Response) => {
    try {
        if (!prover) {
            return res.status(500).json({ error: 'Prover not initialized' });
        }

        const { prevState, newState, originClass, timestamp } = req.body;

        if (!prevState || !newState || originClass === undefined) {
            return res.status(400).json({ error: 'Missing required fields' });
        }

        const proof = await prover.generateProof({
            prevStateHash: prevState,
            newStateHash: newState,
            originClass,
            timestamp: timestamp ?? Math.floor(Date.now() / 1000)
        });

        res.json({
            proof: {
                pA: proof.pA,
                pB: proof.pB,
                pC: proof.pC,
                publicSignals: proof.publicSignals
            },
            metadata: {
                generatedAt: Date.now(),
                circuitVersion: '1.0.0'
            }
        });
    } catch (error) {
        res.status(500).json({
            error: 'Proof generation failed',
            details: getErrorMessage(error)
        });
    }
});

/**
 * Verify proof locally
 */
app.post('/api/v1/verify', (req: Request, res: Response) => {
    try {
        if (!prover) {
            return res.status(500).json({ error: 'Prover not initialized' });
        }

        const { proof, publicSignals } = req.body;

        const isValid = prover.verifyProof(proof, publicSignals);

        res.json({
            valid: isValid,
            verifiedAt: Date.now()
        });
    } catch (error) {
        res.status(500).json({
            error: 'Verification failed',
            details: getErrorMessage(error)
        });
    }
});

/**
 * Get statistics
 */
app.get('/api/v1/stats', (_req: Request, res: Response) => {
    try {
        if (!indexer) {
            return res.status(500).json({ error: 'Indexer not initialized' });
        }

        const stats = indexer.getStats();

        res.json({
            totalTransitions: stats.totalTransitions,
            maxDepth: stats.maxDepth,
            uniqueStates: stats.uniqueStates,
            lastIndexedBlock: stats.lastBlock
        });
    } catch (error) {
        res.status(500).json({
            error: 'Failed to fetch stats',
            details: getErrorMessage(error)
        });
    }
});

/**
 * Start server
 */
export function startServer(
    port: number,
    idx: LineageIndexer,
    prv: LineageProver
) {
    indexer = idx;
    prover = prv;

    app.listen(port, () => {
        console.log(`ZK-ORIGIN API running on port ${port}`);
    });
}