const express = require('express');
const bodyParser = require('body-parser');
const cors = require('cors');
const net = require('net');

const app = express();
const PORT = 3001;
const DB_HOST = '127.0.0.1';
const DB_PORT = 5432;

app.use(cors());
app.use(bodyParser.json());

// Helper to communicate with FerzDB TCP Server
function sendToDb(payload, isHandshake = false) {
    return new Promise((resolve, reject) => {
        const client = new net.Socket();

        client.connect(DB_PORT, DB_HOST, () => {
            client.write(payload);
        });

        client.on('data', (data) => {
            const response = data.toString();
            client.destroy(); // Close connection after response
            resolve(response);
        });

        client.on('error', (err) => {
            reject(err);
        });

        // Timeout after 5 seconds
        setTimeout(() => {
            client.destroy();
            reject(new Error('Connection timeout'));
        }, 5000);
    });
}

// Endpoint for Authentication
app.post('/api/auth', async (req, res) => {
    const { user, pass, db } = req.body;
    const handshake = JSON.stringify({ user, pass, db: db || null });

    try {
        const result = await sendToDb(handshake, true);
        if (result.includes('AUTH_OK')) {
            res.json({ success: true, message: 'Authenticated' });
        } else {
            res.status(401).json({ success: false, message: 'Invalid credentials' });
        }
    } catch (err) {
        res.status(500).json({ success: false, message: err.message });
    }
});

// Endpoint for SQL Execution
app.post('/api/query', async (req, res) => {
    const { user, pass, db, sql } = req.body;

    try {
        // We perform handshake + query in sequence since HTTP is stateless
        const client = new net.Socket();
        client.connect(DB_PORT, DB_HOST, () => {
            // 1. Send Handshake
            client.write(JSON.stringify({ user, pass, db }));

            client.once('data', (data) => {
                const authRes = data.toString();
                if (authRes.includes('AUTH_OK')) {
                    // 2. Send SQL
                    client.write(sql);
                    client.once('data', (queryData) => {
                        const queryRes = queryData.toString();
                        client.destroy();
                        res.json({ success: true, data: queryRes });
                    });
                } else {
                    client.destroy();
                    res.status(401).json({ success: false, message: 'Auth failed in query context' });
                }
            });
        });

        client.on('error', (err) => {
            res.status(500).json({ success: false, message: err.message });
        });

    } catch (err) {
        res.status(500).json({ success: false, message: err.message });
    }
});

app.listen(PORT, () => {
    console.log(`FerzAdmin Bridge running at http://localhost:${PORT}`);
});
