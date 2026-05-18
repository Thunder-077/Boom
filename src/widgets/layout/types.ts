export interface RailItem {
  key: string;
  label: string;
  icon: "person" | "badge" | "domain" | "event_note" | "calendar_month" | "school";
}

export interface SecondaryNavItem {
  key: string;
  label: string;
  icon?: "assignment" | "badge" | "settings" | "inventory_2" | "shuffle" | "tune" | "palette" | "calendar_month" | "published_with_changes" | "query_stats" | "system_update";
}
