import React, { useState } from 'react';
import axios from 'axios';
import { Database, Lock, User, Server } from 'lucide-react';

export default function Login({ onLogin }) {
    const [formData, setFormData] = useState({
        ip: '127.0.0.1:5432',
        user: 'admin',
        pass: 'ferz123'
    });
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState('');

    const handleLogin = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError('');

        try {
            const res = await axios.post('http://localhost:3001/api/auth', formData);
            if (res.data.success) {
                onLogin(formData);
            }
        } catch (err) {
            setError(err.response?.data?.message || 'Connection failed');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="flex items-center justify-center min-h-screen bg-gradient-to-br from-[#0a0a0c] to-[#121218] p-6">
            <div className="glass w-full max-w-md p-8 rounded-2xl">
                <div className="flex flex-col items-center mb-8">
                    <div className="bg-indigo-600/20 p-4 rounded-xl mb-4">
                        <Database className="w-12 h-12 text-indigo-500" />
                    </div>
                    <h1 className="text-3xl font-bold bg-gradient-to-r from-white to-gray-400 bg-clip-text text-transparent">
                        FerzAdmin
                    </h1>
                    <p className="text-gray-400 mt-2">Server Management Console</p>
                </div>

                <form onSubmit={handleLogin} className="space-y-4">
                    <div>
                        <label className="block text-sm font-medium text-gray-400 mb-1">Server IP</label>
                        <div className="relative">
                            <Server className="absolute left-3 top-2.5 w-5 h-5 text-gray-500" />
                            <input
                                type="text"
                                className="input-field w-full pl-10"
                                value={formData.ip}
                                onChange={(e) => setFormData({ ...formData, ip: e.target.value })}
                            />
                        </div>
                    </div>

                    <div className="grid grid-cols-2 gap-4">
                        <div>
                            <label className="block text-sm font-medium text-gray-400 mb-1">Username</label>
                            <div className="relative">
                                <User className="absolute left-3 top-2.5 w-5 h-5 text-gray-500" />
                                <input
                                    type="text"
                                    className="input-field w-full pl-10"
                                    value={formData.user}
                                    onChange={(e) => setFormData({ ...formData, user: e.target.value })}
                                />
                            </div>
                        </div>
                        <div>
                            <label className="block text-sm font-medium text-gray-400 mb-1">Password</label>
                            <div className="relative">
                                <Lock className="absolute left-3 top-2.5 w-5 h-5 text-gray-500" />
                                <input
                                    type="password"
                                    className="input-field w-full pl-10"
                                    value={formData.pass}
                                    onChange={(e) => setFormData({ ...formData, pass: e.target.value })}
                                />
                            </div>
                        </div>
                    </div>

                    {error && <p className="text-red-400 text-sm animate-pulse">{error}</p>}

                    <button type="submit" className="btn-primary w-full mt-4 flex justify-center items-center" disabled={loading}>
                        {loading ? 'Connecting...' : 'Login to Server'}
                    </button>
                </form>
            </div>
        </div>
    );
}
