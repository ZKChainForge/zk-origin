import { ethers } from 'ethers';
import Database from 'better-sqlite3';

interface LineageEvent {
    blockNumber: number;
    transactionHash: string;
    prevStateHash: string;
    newStateHash: string;
    lineageCommitment: string;
    depth: number;
    timestamp: number;
}

export class LineageIndexer {
    private provider: ethers.Provider;
    private contract: ethers.Contract;
    private db: Database;

    constructor(
        rpcUrl: string,
        contractAddress: string,
        abi: any[],
        dbPath: string
    ) {
        this.provider = new ethers.JsonRpcProvider(rpcUrl);
        this.contract = new ethers.Contract(contractAddress, abi, this.provider);
        this.db = new Database(dbPath);
        this.initDatabase();
    }

    private initDatabase(): void {
        this.db.exec(`
            CREATE TABLE IF NOT EXISTS lineage_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                block_number INTEGER NOT NULL,
                transaction_hash TEXT NOT NULL,
                prev_state_hash TEXT NOT NULL,
                new_state_hash TEXT NOT NULL,
                lineage_commitment TEXT NOT NULL,
                depth INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_state_hash ON lineage_events(new_state_hash);
            CREATE INDEX IF NOT EXISTS idx_lineage ON lineage_events(lineage_commitment);
            CREATE INDEX IF NOT EXISTS idx_depth ON lineage_events(depth);
            CREATE INDEX IF NOT EXISTS idx_block ON lineage_events(block_number);
        `);
    }

    async indexFromBlock(startBlock: number): Promise<void> {
        const currentBlock = await this.provider.getBlockNumber();

        console.log(`Indexing from block ${startBlock} to ${currentBlock}`);

        const filter = this.contract.filters.LineageVerified();
        const events = await this.contract.queryFilter(filter, startBlock, currentBlock);

        const insert = this.db.prepare(`
            INSERT INTO lineage_events 
            (block_number, transaction_hash, prev_state_hash, new_state_hash, lineage_commitment, depth, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        `);

        const insertMany = this.db.transaction((events: any[]) => {
            for (const event of events) {
                const block = event.blockNumber;
                const txHash = event.transactionHash;
                const [prevState, newState, lineage, depth] = event.args;

                insert.run(
                    block,
                    txHash,
                    prevState,
                    newState,
                    lineage,
                    Number(depth),
                    Math.floor(Date.now() / 1000)
                );
            }
        });

        insertMany(events);
        console.log(`Indexed ${events.length} events`);
    }

    getLineageHistory(stateHash: string): LineageEvent[] {
        const stmt = this.db.prepare(`
            WITH RECURSIVE lineage_chain AS (
                SELECT * FROM lineage_events WHERE new_state_hash = ?
                UNION ALL
                SELECT e.* FROM lineage_events e
                INNER JOIN lineage_chain lc ON e.new_state_hash = lc.prev_state_hash
            )
            SELECT * FROM lineage_chain ORDER BY depth ASC
        `);

        return stmt.all(stateHash) as LineageEvent[];
    }

    getStatesByDepth(minDepth: number, maxDepth: number): LineageEvent[] {
        const stmt = this.db.prepare(`
            SELECT * FROM lineage_events 
            WHERE depth >= ? AND depth <= ?
            ORDER BY depth ASC
        `);

        return stmt.all(minDepth, maxDepth) as LineageEvent[];
    }

    async watchForEvents(): Promise<void> {
        console.log('Watching for new LineageVerified events...');

        this.contract.on(
            'LineageVerified',
            (prevState, newState, lineage, depth, event) => {
                console.log(`New lineage verified: ${newState} at depth ${depth}`);

                const insert = this.db.prepare(`
                    INSERT INTO lineage_events 
                    (block_number, transaction_hash, prev_state_hash, new_state_hash, lineage_commitment, depth, timestamp)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                `);

                insert.run(
                    event.blockNumber,
                    event.transactionHash,
                    prevState,
                    newState,
                    lineage,
                    Number(depth),
                    Math.floor(Date.now() / 1000)
                );
            }
        );
    }

    /**
     * ✅ Added getStats() method
     */
    getStats(): {
        totalTransitions: number;
        maxDepth: number;
        uniqueStates: number;
        lastBlock: number;
    } {
        const total = this.db
            .prepare(`SELECT COUNT(*) as count FROM lineage_events`)
            .get() as { count: number };

        const maxDepth = this.db
            .prepare(`SELECT MAX(depth) as maxDepth FROM lineage_events`)
            .get() as { maxDepth: number | null };

        const uniqueStates = this.db
            .prepare(`SELECT COUNT(DISTINCT new_state_hash) as count FROM lineage_events`)
            .get() as { count: number };

        const lastBlock = this.db
            .prepare(`SELECT MAX(block_number) as lastBlock FROM lineage_events`)
            .get() as { lastBlock: number | null };

        return {
            totalTransitions: total.count ?? 0,
            maxDepth: maxDepth.maxDepth ?? 0,
            uniqueStates: uniqueStates.count ?? 0,
            lastBlock: lastBlock.lastBlock ?? 0
        };
    }
}