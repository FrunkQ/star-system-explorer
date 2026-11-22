// A tiny Mag layer fixture for views to build and test against before read_field's Solar System fixture lands
// (MF14a part 1b swaps it for fieldFromRead). Test data, not physics: Earth's dipole from the measured-fields
// fixture, the kernel's stand-off for Earth from read_light (9.3 radii), Shue's flaring for a quiet wind, and a
// Parker-spiral wind field of about 5 nT in an away sector.
import { readFileSync } from "node:fs";
import { dipole, type Bubble, type FieldModel, type V3 } from "../src/field.ts";

const measured = JSON.parse(readFileSync(new URL("../../../tests/fixtures/magnetic/measured-fields.json", import.meta.url), "utf8")) as {
  bodies: { name: string; reference_radius_m: number; dipole_surface_equatorial_t: number; tilt_deg: number }[];
};

export function earthField(): { model: FieldModel; bubble: Bubble } {
  const e = measured.bodies.find((b) => b.name === "Earth")!;
  // Earth's north magnetic pole leans toward about 290°E; the fixture does not hold the direction yet.
  const model = dipole(e.reference_radius_m, e.dipole_surface_equatorial_t, e.tilt_deg, 290);
  const bubble: Bubble = { standoff_radii: 9.3, flaring: 0.58, sunward: [1, 0, 0], wind_field_t: [-3.5e-9, -3.5e-9, 0] as V3 };
  return { model, bubble };
}
