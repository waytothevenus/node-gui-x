// import { useState } from "react";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import Home from "./pages/Home";
import "./App.css";
import Init from "./pages/Init";

function App() {
  return (
    <Router>
      <header></header>
      <main>
        <Routes>
          <Route path="/" element={<Init />} />
          <Route path="/home" element={<Home />} />
        </Routes>
      </main>
      <footer></footer>
    </Router>
  );
}

export default App;
