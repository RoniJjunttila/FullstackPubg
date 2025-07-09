import React, { useEffect, useState } from "react";
import "../styles/Match.css";
import { useParams } from "react-router-dom";

function Match({ selectedMatch }) {
  const [matchData, setMatchData] = useState([]);
  const [currentMatch, setCurrentMatch] = useState();
  const [bShowBluezoneDamage, setBshowBluezoneDamage] = useState(false);
  const [bShowBleedOutDamage, setBshowBleedOutDamage] = useState(false);

  const { id } = useParams();
  const [currentMapImage, setCurrentMapImage] = useState("");

  useEffect(() => {
    let matchId;

    if (selectedMatch === undefined) {
      if (id === "") return;
      matchId = id;
      fetch("/api/matches")
        .then((response) => response.json())
        .then((json) => {
          const data = json.find((match) => match.id === id);
          setCurrentMatch({
            date: data.date,
            map_name: data.map_name,
            game_mode: data.game_mode,
          });
          console.log("Fetched data from /api/matches:", json);
        })
        .catch((error) => {
          console.error("Error fetching data:", error);
        });
    } else {
      matchId = selectedMatch.id;
      setCurrentMatch(selectedMatch);
    }

    fetch("/api/match/" + matchId)
      .then((response) => response.json())
      .then((json) => {
        console.log("Fetched data from /api/matches:", json);
        setMatchData(json);
      })
      .catch((error) => {
        console.error("Error fetching data:", error);
      });
  }, [selectedMatch, id]);

  useEffect(() => {
    if (!currentMatch) return;

    switch (currentMatch.map_name) {
      case "Erangel":
        setCurrentMapImage("https://i.imgur.com/7Z8dQJD.png");
        break;
      case "Deston":
        setCurrentMapImage("https://i.imgur.com/9Z27WqD.png");
        break;
      case "Miramar":
        setCurrentMapImage("https://i.imgur.com/g8kQVgY.png");
        break;
      case "Taego":
        setCurrentMapImage("https://i.imgur.com/kEZSF3W.png");
        break;
      default:
        setCurrentMapImage("https://i.imgur.com/7Z8dQJD.png");
    }
  }, [currentMatch]);

  const renderEvent = (attack, index) => {
    if (attack.damageTypeCategory === "Fall Damage") {
      return eventTypeFallDamage(attack, index);
    } else if (attack.damageTypeCategory === "Bluezone Damage") {
      if (bShowBluezoneDamage) {
        return eventTypeBluezone(attack, index);
      }
    } else if (attack._T === "LogPlayerKillV2") {
      return eventTypeLogPlayerKill(attack, index);
    } else if (attack._T === "LogPlayerTakeDamage") {
      return eventTypePlayerAttack(attack, index);
    } else {
      return null;
    }
  };

  const eventTypeFallDamage = (attack, index) => {
    return (
      <div key={index} className="attack-details-box">
        <div className="fall-header">
          {attack.victim.name} (Team {attack.victim.teamId}) @{" "}
          {(attack._D / 60).toFixed(2)}
        </div>
        <p className="fall-damage-text">
          <span className="damage-label">Fall Damage</span>{" "}
          <span className="damage-values">
            {attack.victim.health.toFixed(2)} - {attack.damage.toFixed(2)}{" "}
            {"=>"} {(attack.victim.health - attack.damage).toFixed(2)}
          </span>
        </p>
      </div>
    );
  };

  const eventTypeBluezone = (attack, index) => {
    if (attack.victim.health.toFixed(2) === "0.00") return null; // DBNO damage outside zone
    return (
      <div key={index} className="attack-details-box">
        <div className="bluezone-header">
          {attack.victim.name} (Team {attack.victim.teamId}) @{" "}
          {(attack._D / 60).toFixed(2)}
        </div>
        <p className="bluezone-damage-text">
          <span className="bluezone-label">
            Bluezone damaged{" "}
            {attack.repeatCount > 1 && `${attack.repeatCount}x`}
          </span>{" "}
          <span className="bluezone-value">
            {attack.victim.health.toFixed(2) === "0.00"
              ? "(DBNO outside zone)"
              : "to " + attack.victim.health.toFixed(2)}
          </span>
        </p>
      </div>
    );
  };

  const eventTypeLogPlayerKill = (attack, index) => {
    return (
      <div key={index} className="attack-details-box">
        <div className="kill-header">
          <p>
            {attack.killer?.name}
            {attack.killer?.name && " killed "}
            {attack.finishDamageInfo.damageCauserName === "Bluezone" ||
            attack.finishDamageInfo.damageTypeCategory === "Bleed Out Damage"
              ? `(${attack.finishDamageInfo.damageTypeCategory})`
              : attack.finishDamageInfo.damageCauserName === "Bluezone"
                ? ""
                : "with " + attack.finishDamageInfo.damageCauserName}{" "}
            {attack.finishDamageInfo.damageCauserName === "Bluezone"
              ? "killed "
              : " "}
            {attack.victim.name} (Team {attack.victim.teamId})
            {attack.finishDamageInfo.damageCauserName !== "Bluezone" &&
              attack.finishDamageInfo.damageTypeCategory !==
                "Bleed Out Damage" &&
              attack.distance?.toFixed(2) !== -1.0 &&
              ` — distance: ${attack.distance?.toFixed(2)} meters`}
          </p>
        </div>
      </div>
    );
  };

  const eventTypePlayerAttack = (attack, index) => {
    if (attack.attackId === -1) return null;
    if (attack.victim.health.toFixed(2) === "0.00" && !bShowBleedOutDamage)
      return null;
    return (
      <div className="attack-details-box" key={index}>
        <div className="attack-header">
          <p>
            {attack.attacker ? attack.attacker.name : "N/A"} attacked{" "}
            {attack.victim.name} (Team {attack.victim.teamId}) @{" "}
            {(attack._D / 60).toFixed(2)}
          </p>
        </div>
        <div className="attack-body">
          {attack.weapon && (
            <div className="weapon-details">
              <p className="damage-line">
                {attack.weapon.itemId} damaged{" "}
                {attack.damageReason ? attack.damageReason.toLowerCase() : ""},{" "}
                {attack.victim.health.toFixed(2) === "0.00"
                  ? "DBNO damage"
                  : `${attack.victim.health.toFixed(2)} - ${attack.damage ? attack.damage.toFixed(2) : "N/A"} → ` +
                    (attack.victim.health - attack.damage).toFixed(2)}
              </p>

              {attack.weapon.attachedItems.length !== 0 && (
                <p className="attachments-line">
                  Attachments: {attack.weapon.attachedItems.join(", ")}
                </p>
              )}
              
              {attack.helmet && attack.vest && (
                <p className="attachments-line">{attack.attacker.name} armor: 
                 {attack.helmet.item},{" "}
                  {attack.vest.item}
                </p>
              )}

              {attack?.victim_helmet && attack?.victim_vest && (
                <p className="attachments-line">
                  {attack.victim.name} armor: {attack.victim_helmet.item},{" "}
                  {attack.victim_vest.item}
                </p>
              )}

              {attack.distance !== undefined &&
                attack.distance !== null &&
                attack.weapon.itemId !== "Frag Grenade" && (
                  <p className="distance-line">
                    Distance: {attack.distance.toFixed(2)} meters @ speed:{" "}
                    {attack?.bullet_speed === 0 || attack?.bullet_speed === null
                      ? "N/A"
                      : `${attack.bullet_speed.toFixed(0)} m/s`}{" "}
                    with hit bullet number: {attack.fireWeaponStackCount}
                  </p>
                )}
            </div>
          )}
        </div>
      </div>
    );
  };

  const mergedEventsPerUser = {};

  matchData.forEach((entry) => {
    const user = entry.victim.name;

    if (!mergedEventsPerUser[user]) {
      mergedEventsPerUser[user] = [];
    }

    const lastEvent =
      mergedEventsPerUser[user][mergedEventsPerUser[user].length - 1];

    if (
      lastEvent &&
      lastEvent._T === entry._T &&
      lastEvent.damageTypeCategory === "Bluezone Damage"
    ) {
      lastEvent.damage += entry.damage;
      lastEvent.victim.health = entry.victim.health;
      //   lastEvent._D = lastEvent._D;
    } else {
      mergedEventsPerUser[user].push({ ...entry });
    }
  });

  const usersWithEvents = Object.values(mergedEventsPerUser)
    .filter((value) => Array.isArray(value))
    .flat();

  const sorted = usersWithEvents.sort(
    (a, b) => parseFloat(a._D) - parseFloat(b._D),
  );

  if (false) {
    return (
      <div className="match-details-container">
        <p className="header-text">Loading</p>
        <div className="image-wrapper">
          <div className="image-padding">
            <div
              style={{
                maxWidth: "700px",
                margin: "0 auto",
              }}
            >
              {Array.from({ length: 20 }).map((_, index) => (
                <div key={index} className="attack-details-box">
                  <div className="fall-header">Loading</div>
                  <p className="fall-damage-text">
                    <span className="damage-label">Loading</span>{" "}
                    <span className="damage-values"></span>
                  </p>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    );
  } else {
    return (
      <div className="match-details-container">
        <p className="header-text">
          {currentMatch?.map_name}, {currentMatch?.game_mode},{" "}
          {new Date(currentMatch?.date).toLocaleString("en-Us", {
            day: "numeric",
            month: "long",
            hour: "2-digit",
            minute: "2-digit",
            hour12: false,
            timeZone: "Europe/Helsinki",
          })}
        </p>
        <div className="image-wrapper">
          <img src={currentMapImage} alt="Header" className="header-image" />
          <div
            style={{ display: "flex", gap: "10px", justifyContent: "center" }}
          >
            <button
              className="ghibli-button"
              title="show bleed out damage"
              onClick={() => setBshowBleedOutDamage((prev) => !prev)}
              style={{
                backgroundColor: bShowBleedOutDamage ? "#ccc" : "#fff",
                width: "25%",
              }}
            >
              Bleed Out
            </button>
            <button
              className="ghibli-button"
              title="show bluezone damage"
              onClick={() => setBshowBluezoneDamage((prev) => !prev)}
              style={{
                backgroundColor: bShowBluezoneDamage ? "#ccc" : "#fff",
                width: "25%",
              }}
            >
              Bluezone
            </button>
          </div>
          <div className="image-padding">
            <div
              style={{
                maxWidth: "700px",
                margin: "0 auto",
              }}
            >
              {matchData.map((attack, index) => renderEvent(attack, index))}
            </div>
          </div>
        </div>
      </div>
    );
  }
}

export default Match;
