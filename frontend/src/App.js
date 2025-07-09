import React from "react";
import { BrowserRouter as Router, Routes, Route, Link } from "react-router-dom";
import Home from "./pages/Home";
import MatchDetails from "./pages/MatchDetails";
import Footer from "./components/Footer";
import "./App.css";
import Header from "./components/Header";

function App() {
  return (
    <Router>
      <div className="app">
        <Header />
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/match/:matchId" element={<MatchDetails />} />
        </Routes>
        <Footer />
      </div>
    </Router>
  );
}

export default App;
