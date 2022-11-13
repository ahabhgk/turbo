import * as React from "react";
import styles from "./Button.module.css";

export default function Button({ children, onClick }) {
  return (
    <button onClick={onClick} className={styles.btn}>
      {children}
    </button>
  );
}
