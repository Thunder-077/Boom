export interface RailItem {
  key: string;
  label: string;
  icon: "school" | "co_present" | "domain" | "event_note";
}

export interface SecondaryNavItem {
  key: string;
  label: string;
  icon?: "assignment" | "badge" | "settings" | "inventory_2" | "shuffle" | "tune";
}
