import React from 'react'
import { Routes, Route, NavLink } from 'react-router-dom'
import Dashboard from './pages/Dashboard'
import ReviewDetail from './pages/ReviewDetail'
import RulesManager from './pages/RulesManager'
import Evaluation from './pages/Evaluation'

const App: React.FC = () => {
  return (
    <div className="app">
      <nav className="sidebar">
        <div className="logo">
          <h2>Code Review Agent</h2>
          <span className="version">v0.1.0</span>
        </div>
        <ul className="nav-links">
          <li><NavLink to="/" end>Dashboard</NavLink></li>
          <li><NavLink to="/rules">Rules Manager</NavLink></li>
          <li><NavLink to="/evaluation">Evaluation</NavLink></li>
        </ul>
        <div className="sidebar-footer">
          <span>Rust + Python + LLM</span>
        </div>
      </nav>
      <main className="content">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/review/:id" element={<ReviewDetail />} />
          <Route path="/rules" element={<RulesManager />} />
          <Route path="/evaluation" element={<Evaluation />} />
        </Routes>
      </main>
    </div>
  )
}

export default App
