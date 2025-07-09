import React from "react";
import { useNavigate } from "react-router-dom";
import ".././App.css";

function Header() {
  const navigate = useNavigate();
  return (
    <div onClick={() => navigate("/")} className="header">
      <div className="header-container">
        <p>Pubg stats</p>
      </div>
    </div>
  );
}

export default Header;
