import React from "react";
import Matches from "../components/Matches";
import { useNavigate } from "react-router-dom";

function Home() {
  const navigate = useNavigate();

  const handleChangeSelectedMatch = (match) => {
    navigate(`/match/${match.id}`, { state: { match } });
  };

  return (
    <div>
      <Matches handleChangeSelectedMatch={handleChangeSelectedMatch} />
    </div>
  );
}

export default Home;
