import React, { useEffect, useState } from "react";
import headerImage from "../img/pubg_gbili.png";
import "../styles/Matches.css";

function Matches({ handleChangeSelectedMatch }) {
  const [matches, setMatches] = useState([]);
  const [extendDetails, setExtendDetails] = useState([]);
  const [extendDetailsAnimation, setExtendDetailsAnimation] = useState([]);

  useEffect(() => {
    fetch("/api/matches")
      .then((response) => response.json())
      .then((json) => {
        console.log("Fetched data from /api/matches:", json);
        setMatches(json);
      })
      .catch((error) => {
        console.error("Error fetching data:", error);
      });
  }, []);

  const updateExtensionRender = (collapse, index) => {
    setExtendDetailsAnimation((prev) => {
      const updated = [...prev];
      if (collapse) {
        return updated.filter((i) => i !== index);
      } else {
        if (!updated.includes(index)) updated.push(index);
        return updated;
      }
    });
    setTimeout(() => {
      setExtendDetails((prev) => {
        const updated = [...prev];
        if (collapse) {
          return updated.filter((i) => i !== index);
        } else {
          if (!updated.includes(index)) updated.push(index);
          return updated;
        }
      });
    }, 100);
  };

  const changeSelectedMatch = (match) => {
    handleChangeSelectedMatch(match);
  };

  return (
    <div className="matches-container">
      <div className="container">
        <p className="container-title">Matches</p>
      </div>

      {matches
        .sort((a, b) => new Date(b.date) - new Date(a.date))
        .map((match, index) => {
          const isExpanded = extendDetails.includes(index);
          const isExpandedAnimation = extendDetailsAnimation.includes(index);
          return (
            <div
              key={index}
              className="match-card"
              onClick={() => changeSelectedMatch(match)}
            >
              <div className="match-header">
                <div className="match-info">
                  {`${match.map_name}, ${new Date(match.date).toLocaleString(
                    "en-US",
                    {
                      day: "numeric",
                      month: "long",
                      hour: "2-digit",
                      minute: "2-digit",
                      hour12: false,
                      timeZone: "Europe/Helsinki",
                    },
                  )}`}
                </div>

                <div
                  className={`win-place ${
                    match.squad[0].winPlace === 1 ? "win" : "lose"
                  }`}
                >
                  {match.squad[0].winPlace}
                </div>
              </div>

              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "flex-start",
                }}
              >
                <div
                  className={`slide-container-expand ${
                    isExpanded
                      ? "slide-enter-active-expand"
                      : "slide-exit-active-expand"
                  }`}
                >
                  <div
                    className="players-grid"
                    style={{
                      gridTemplateColumns: isExpanded
                        ? "1fr"
                        : "repeat(4, 0.5fr)",
                    }}
                  >
                    {!isExpanded ? (
                      match.squad.map((player, idx) => (
                        <div key={idx} className="player-condensed">
                          <span className="player-name">
                            {"|"} {player.name} {"|"}
                          </span>
                        </div>
                      ))
                    ) : (
                      <div
                        className={`slide-container ${
                          isExpandedAnimation
                            ? "slide-enter-active"
                            : "slide-exit-active"
                        }`}
                      >
                        <table className="stats-table">
                          <thead>
                            <tr>
                              <th>Name</th>
                              <th>Kills</th>
                              <th>Damage dealt</th>
                              <th>DBNOs</th>
                              <th>Assists</th>
                              <th>Ride distance</th>
                            </tr>
                          </thead>
                          <tbody>
                            {match.squad.map((player, idx) => (
                              <tr key={idx}>
                                <td>{player.name}</td>
                                <td>{Math.round(player.kills)}</td>
                                <td>{Math.round(player.damageDealt)}</td>
                                <td>{player.DBNOs}</td>
                                <td>{player.assists}</td>
                                <td>
                                  {(player.rideDistance / 1000).toFixed(2)} km
                                </td>
                              </tr>
                            ))}
                          </tbody>
                        </table>
                      </div>
                    )}
                  </div>
                </div>
              </div>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  updateExtensionRender(isExpanded, index);
                }}
                className="ghibli-button"
                title={isExpanded ? "minimize" : "extend"}
              >
                {isExpanded ? "Collapse" : "Expand"}
              </button>
            </div>
          );
        })}
    </div>
  );
}

export default Matches;
