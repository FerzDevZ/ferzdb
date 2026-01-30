import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import Login from './components/Login';
import Dashboard from './components/Dashboard';

export default function App() {
  const [auth, setAuth] = useState(null); // { user, pass, db }

  return (
    <div className="min-h-screen">
      <AnimatePresence mode="wait">
        {!auth ? (
          <motion.div
            key="login"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
          >
            <Login onLogin={setAuth} />
          </motion.div>
        ) : (
          <motion.div
            key="dashboard"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
          >
            <Dashboard auth={auth} onLogout={() => setAuth(null)} />
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
