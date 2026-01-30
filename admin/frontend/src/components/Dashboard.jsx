import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { LogOut, Play, Code, Database, Table, Plus, Server, Layers } from 'lucide-react';

export default function Dashboard({ auth, onLogout }) {
    const [databases, setDatabases] = useState([]);
    const [tables, setTables] = useState([]);
    const [selectedDb, setSelectedDb] = useState(auth.db || 'none');
    const [sql, setSql] = useState('SELECT * FROM users');
    const [results, setResults] = useState(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState('');

    useEffect(() => {
        fetchDatabases();
    }, []);

    const fetchDatabases = async () => {
        try {
            const res = await axios.post('http://localhost:3001/api/query', { ...auth, db: selectedDb, sql: 'SHOW DATABASES' });
            if (res.data.success) {
                const dbListMatch = res.data.data.match(/\[.*\]/);
                if (dbListMatch) setDatabases(JSON.parse(dbListMatch[0]));
            }
        } catch (err) { console.error(err); }
    };

    const fetchTables = async (dbName) => {
        try {
            const res = await axios.post('http://localhost:3001/api/query', { ...auth, db: dbName, sql: 'SHOW TABLES' });
            if (res.data.success) {
                // Parse "Tables in main:\n[\"users\", \"orders\"]"
                const tableListMatch = res.data.data.match(/\[.*\]/);
                if (tableListMatch) setTables(JSON.parse(tableListMatch[0]));
                else setTables([]);
            }
        } catch (err) { console.error('Failed to fetch tables', err); }
    };

    const switchDatabase = async (dbName) => {
        setLoading(true);
        try {
            const res = await axios.post('http://localhost:3001/api/query', { ...auth, db: dbName, sql: `USE ${dbName}` });
            if (res.data.success) {
                setSelectedDb(dbName);
                fetchTables(dbName);
                setResults(`Switched to: ${dbName}`);
            }
        } catch (err) { setError(`Failed to switch to ${dbName}`); } finally { setLoading(false); }
    };

    const browseTable = (tableName) => {
        const query = `SELECT * FROM ${tableName}`;
        setSql(query);
        runQuery(query);
    };

    const runQuery = async (queryOverride) => {
        const queryToRun = queryOverride || sql;
        setLoading(true);
        setError('');
        try {
            const res = await axios.post('http://localhost:3001/api/query', { ...auth, db: selectedDb, sql: queryToRun });
            if (res.data.success) {
                setResults(res.data.data);
                if (queryToRun.toUpperCase().includes('CREATE DATABASE') || queryToRun.toUpperCase().includes('USE')) {
                    fetchDatabases();
                }
                if (queryToRun.toUpperCase().includes('CREATE TABLE') || queryToRun.toUpperCase().includes('DROP')) {
                    fetchTables(selectedDb);
                }
            }
        } catch (err) {
            setError(err.response?.data?.message || 'Execution error');
        } finally { setLoading(false); }
    };

    const parseResultSet = (raw) => {
        if (!raw || typeof raw !== 'string') return null;
        if (!raw.includes('Columns:')) return { text: raw };

        try {
            const lines = raw.split('\n');
            const colLine = lines.find(l => l.startsWith('Columns:'));
            if (!colLine) return { text: raw };

            const columns = JSON.parse(colLine.replace('Columns: ', ''));
            const rows = lines
                .filter(l => l.startsWith('[') && !l.startsWith('Columns:'))
                .map(l => {
                    // Parse "[Int(1), Text("admin")]" -> ["1", "admin"]
                    return l.slice(1, -1).split(', ').map(v => {
                        return v.replace(/^[A-Za-z]+\((.*)\)$/, '$1').replace(/^"(.*)"$/, '$1');
                    });
                });

            return { columns, rows };
        } catch (e) {
            return { text: raw };
        }
    };

    const ResultDisplay = () => {
        if (error) return (
            <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-xl text-red-400 text-sm flex items-center gap-2">
                <div className="w-1.5 h-1.5 rounded-full bg-red-500" /> {error}
            </div>
        );
        if (!results) return (
            <div className="flex flex-col items-center justify-center py-20 text-gray-700">
                <Play className="w-12 h-12 mb-4 opacity-10" />
                <p className="text-sm font-medium">Execute a query to see results here</p>
            </div>
        );

        const data = parseResultSet(results);

        if (data.columns && data.rows) {
            return (
                <div className="overflow-x-auto">
                    <table className="w-full text-left text-sm border-collapse">
                        <thead>
                            <tr className="bg-white/5 border-b border-white/10">
                                {data.columns.map(col => (
                                    <th key={col} className="px-6 py-3 font-bold text-gray-400 uppercase tracking-widest text-[10px]">{col}</th>
                                ))}
                            </tr>
                        </thead>
                        <tbody>
                            {data.rows.map((row, i) => (
                                <tr key={i} className="border-b border-white/5 hover:bg-white/[0.02] transition-colors">
                                    {row.map((val, j) => (
                                        <td key={j} className="px-6 py-4 font-mono text-indigo-300">{val}</td>
                                    ))}
                                </tr>
                            ))}
                        </tbody>
                    </table>
                    {data.rows.length === 0 && (
                        <p className="p-8 text-center text-gray-600 text-xs italic">Empty result set</p>
                    )}
                </div>
            );
        }

        return (
            <pre className="text-xs font-mono text-indigo-200 bg-black/40 p-6 rounded-xl border border-white/5 overflow-x-auto leading-relaxed">
                {data.text}
            </pre>
        );
    };

    return (
        <div className="flex h-screen bg-[#0a0a0c] text-slate-200">
            {/* Sidebar */}
            <div className="w-72 glass border-r border-white/5 flex flex-col">
                {/* ... Sidebar remains the same as previous edit ... */}
                <div className="p-6 border-b border-white/5">
                    <div className="flex items-center gap-2 font-bold text-xl">
                        <div className="bg-indigo-500/20 p-2 rounded-lg">
                            <Database className="text-indigo-500 w-5 h-5" />
                        </div>
                        <span>FerzAdmin</span>
                    </div>
                </div>

                <div className="flex-1 overflow-y-auto p-4 space-y-6">
                    <div className="space-y-1">
                        <div className="flex items-center justify-between px-4 mb-2">
                            <p className="text-[10px] text-gray-500 uppercase font-black tracking-widest">Databases</p>
                            <button
                                onClick={() => { const n = prompt('DB Name:'); if (n) runQuery(`CREATE DATABASE ${n}`) }}
                                className="p-1 hover:bg-white/10 rounded transition-colors text-indigo-400"
                            >
                                <Plus className="w-3 h-3" />
                            </button>
                        </div>
                        {databases.map(db => (
                            <div key={db} className="space-y-1">
                                <button
                                    onClick={() => switchDatabase(db)}
                                    className={`flex items-center gap-3 w-full px-4 py-2 rounded-lg transition-all text-sm ${selectedDb === db ? 'bg-indigo-500/20 text-indigo-100 ring-1 ring-indigo-500/30' : 'hover:bg-white/5 text-gray-400'}`}
                                >
                                    <Database className={`w-3.5 h-3.5 ${selectedDb === db ? 'text-indigo-400' : 'text-gray-600'}`} />
                                    {db}
                                </button>
                                {selectedDb === db && (
                                    <div className="ml-8 space-y-1 border-l border-white/10 pl-2 mt-1">
                                        {tables.length > 0 ? tables.map(table => (
                                            <button
                                                key={table}
                                                onClick={() => browseTable(table)}
                                                className="flex items-center gap-2 w-full px-2 py-1.5 rounded-md hover:bg-white/5 text-xs text-gray-400 hover:text-indigo-300 transition-colors"
                                            >
                                                <Table className="w-3 h-3 opacity-50" /> {table}
                                            </button>
                                        )) : <p className="text-[10px] text-gray-600 italic px-2">No tables</p>}
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                </div>

                <div className="p-4 border-t border-white/5">
                    <div className="p-4 glass rounded-xl bg-white/5">
                        <p className="text-[10px] text-gray-500 uppercase font-bold tracking-widest mb-2">Connected to</p>
                        <div className="flex items-center gap-2">
                            <Server className="w-3.5 h-3.5 text-indigo-400" />
                            <span className="text-sm font-medium">{auth.ip}</span>
                        </div>
                    </div>
                    <button onClick={onLogout} className="flex items-center gap-2 w-full mt-4 px-4 py-2 rounded-lg text-red-500/80 hover:bg-red-500/10 transition-colors text-sm">
                        <LogOut className="w-4 h-4" /> Sign Out
                    </button>
                </div>
            </div>

            {/* Main Content */}
            <div className="flex-1 flex flex-col overflow-hidden">
                <header className="h-16 flex items-center justify-between px-8 bg-black/20 border-b border-white/5 backdrop-blur-md">
                    <div className="flex items-center gap-4">
                        <h2 className="text-sm font-medium text-gray-400 flex items-center gap-2">
                            <Layers className="w-4 h-4" /> Explorer
                        </h2>
                        <div className="h-4 w-px bg-white/10" />
                        <span className="text-xs text-indigo-400 font-bold bg-indigo-500/10 px-3 py-1 rounded-full border border-indigo-500/20 truncate max-w-[200px]">
                            {selectedDb !== 'none' ? `db: ${selectedDb}` : 'Select a DB'}
                        </span>
                    </div>
                    <div className="flex items-center gap-4">
                        <span className="text-[10px] px-3 py-1.5 rounded-full bg-emerald-500/5 text-emerald-500 border border-emerald-500/10 font-black uppercase tracking-tighter">
                            Online
                        </span>
                    </div>
                </header>

                <div className="flex-1 p-8 overflow-y-auto space-y-6 bg-[#0c0c0e]">
                    <div className="glass rounded-xl overflow-hidden border border-white/5">
                        <div className="flex items-center justify-between px-6 py-3 border-b border-white/5 bg-white/[0.02]">
                            <div className="flex items-center gap-4">
                                <Code className="w-4 h-4 text-gray-500" />
                                <div className="flex gap-2">
                                    {['SELECT', 'INSERT', 'CREATE', 'SHOW'].map(cmd => (
                                        <button key={cmd} onClick={() => setSql(s => `${cmd} ${s}`)} className="text-[10px] font-bold text-gray-500 hover:text-indigo-400 px-2 py-0.5 rounded border border-white/5 transition-all">{cmd}</button>
                                    ))}
                                </div>
                            </div>
                            <button onClick={() => runQuery()} disabled={loading} className="btn-primary py-1.5 px-6 rounded-lg text-xs flex items-center gap-2">
                                {loading ? 'Running...' : <><Play className="w-3 h-3 fill-current" /> Execute</>}
                            </button>
                        </div>
                        <textarea
                            className="w-full h-32 bg-transparent p-6 font-mono text-indigo-300 focus:outline-none transition-all resize-none text-sm"
                            value={sql} onChange={(e) => setSql(e.target.value)}
                        />
                    </div>

                    <div className="glass rounded-xl overflow-hidden border border-white/5 shadow-2xl">
                        <div className="px-6 py-3 border-b border-white/5 flex items-center justify-between bg-white/[0.02]">
                            <h3 className="font-bold text-gray-500 text-[10px] uppercase tracking-widest">Results</h3>
                        </div>
                        <div className="p-0">
                            <ResultDisplay />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
