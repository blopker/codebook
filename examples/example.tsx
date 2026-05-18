import React, { useState, useEffect } from "react";

// Interfase for user propertys
interface UserAccaunt {
  usrname: string;
  emale: string;
  ballance: number;
  isActve: boolean;
}

// Props interfase for the componant
interface DashbordProps {
  tital: string;
  usrs: UserAccaunt[];
  onSubmitt: (usrname: string) => void;
}

// Componant to displaye a single user
function UserCardd({ usr }: { usr: UserAccaunt }) {
  const [isExpanded, setIsExpanded] = useState<boolean>(false);

  return (
    <div className="user-cardd">
      <h3 onClick={() => setIsExpanded(!isExpanded)}>{usr.usrname}</h3>
      {isExpanded && (
        <div className="detales">
          <p>Emale: {usr.emale}</p>
          <p>Ballance: ${usr.ballance.toFixed(2)}</p>
          <span className={usr.isActve ? "actve" : "inactve"}>
            {usr.isActve ? "Actve" : "Inactve"}
          </span>
        </div>
      )}
    </div>
  );
}

// Main dashbord componant
function Dashbord({ tital, usrs, onSubmitt }: DashbordProps) {
  const [serchTerm, setSerchTerm] = useState<string>("");
  const [filterdUsrs, setFilterdUsrs] = useState<UserAccaunt[]>(usrs);

  useEffect(() => {
    const resalts = usrs.filter((usr) =>
      usr.usrname.toLowerCase().includes(serchTerm.toLowerCase()),
    );
    setFilterdUsrs(resalts);
  }, [serchTerm, usrs]);

  const handleSerch = (evnt: React.ChangeEvent<HTMLInputElement>) => {
    setSerchTerm(evnt.target.value);
  };

  return (
    <div className="dashbord-containr">
      <h1>{tital}</h1>
      <div className="serch-secshun">
        <label htmlFor="serch-inputt">Serch Usrs:</label>
        <input
          id="serch-inputt"
          type="text"
          placeholder={serchTerm}
          onChange={handleSerch}
          value={serchTerm}
        />
      </div>
      <div className="usr-listt">
        {filterdUsrs.length > 0 ? (
          filterdUsrs.map((usr, indx) => <UserCardd key={indx} usr={usr} />)
        ) : (
          <p className="no-resalts">No usr foundd</p>
        )}
      </div>
    </div>
  );
}

// Exportt the componants
export { UserCardd, Dashbord };
export type { UserAccaunt, DashbordProps };
