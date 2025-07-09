import React from "react";
import { useParams, useLocation } from "react-router-dom";
import Match from "../components/Match";

function MatchDetails() {
  const { matchId } = useParams();
  const location = useLocation();
  const match = location.state?.match;
  return (
    <div>
      <Match selectedMatch={match} />
    </div>
  );
}

export default MatchDetails;
