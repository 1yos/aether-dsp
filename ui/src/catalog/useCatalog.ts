import { useState, useCallback } from "react";
import { CATALOG, CATALOG_REGIONS } from "./catalogData";
import type {
  CatalogInstrument,
  CatalogRegion,
  InstrumentFamily,
  CustomCatalogEntry,
  Region,
} from "./types";

const CUSTOM_KEY = "aether_custom_catalog";

function loadCustom(): CustomCatalogEntry[] {
  try {
    const raw = localStorage.getItem(CUSTOM_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveCustom(entries: CustomCatalogEntry[]) {
  localStorage.setItem(CUSTOM_KEY, JSON.stringify(entries));
}

export function useCatalog() {
  const [customInstruments, setCustomInstruments] =
    useState<CustomCatalogEntry[]>(loadCustom);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedRegion, setSelectedRegion] = useState<Region | null>(null);

  const allInstruments: CatalogInstrument[] = [
    ...CATALOG,
    ...customInstruments,
  ];

  const regions: CatalogRegion[] = [...CATALOG_REGIONS] as CatalogRegion[];

  const families: InstrumentFamily[] = Array.from(
    new Set(allInstruments.map((i) => i.family)),
  ) as InstrumentFamily[];

  const search = useCallback(
    (query: string): CatalogInstrument[] => {
      if (!query.trim()) return allInstruments;
      const q = query.toLowerCase();
      return allInstruments.filter(
        (i) =>
          i.name.toLowerCase().includes(q) ||
          i.region.toLowerCase().includes(q) ||
          i.country.toLowerCase().includes(q) ||
          i.description.toLowerCase().includes(q) ||
          i.tags.some((t) => t.toLowerCase().includes(q)),
      );
    },
    [allInstruments],
  );

  const filterByRegion = useCallback(
    (region: string): CatalogInstrument[] =>
      allInstruments.filter((i) => i.region === region),
    [allInstruments],
  );

  const filterByFamily = useCallback(
    (family: string): CatalogInstrument[] =>
      allInstruments.filter((i) => i.family === family),
    [allInstruments],
  );

  const getById = useCallback(
    (id: string): CatalogInstrument | undefined =>
      allInstruments.find((i) => i.id === id),
    [allInstruments],
  );

  const addCustom = useCallback(
    (entry: Omit<CustomCatalogEntry, "is_custom" | "added_at">) => {
      const full: CustomCatalogEntry = {
        ...entry,
        is_custom: true,
        added_at: new Date().toISOString(),
      };
      setCustomInstruments((prev) => {
        const updated = [...prev, full];
        saveCustom(updated);
        return updated;
      });
    },
    [],
  );

  const removeCustom = useCallback((id: string) => {
    setCustomInstruments((prev) => {
      const updated = prev.filter((i) => i.id !== id);
      saveCustom(updated);
      return updated;
    });
  }, []);

  return {
    instruments: allInstruments,
    builtInCount: CATALOG.length,
    customCount: customInstruments.length,
    regions,
    families,
    search,
    filterByRegion,
    filterByFamily,
    getById,
    addCustom,
    removeCustom,
    searchQuery,
    setSearchQuery,
    selectedRegion,
    setSelectedRegion,
  };
}
