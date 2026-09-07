# `tools/aux-gen` — Auxiliary Property Grids (`FRAUX1`)

This tool tabulates auxiliary thermodynamic and transport property surfaces in `FRAUX1` format for fluids and regions outside standard $(P, h)$ saturation dome geometry:

| Kind | Target Fluids / Regions | Characteristics |
|---|---|---|
| `INCOMPRESSIBLE` | `INCOMP::MEG[x]`, `INCOMP::MPG[x]` (aqueous glycols) | Dome-free incompressible liquids with concentration and temperature axes. |
| `PRESSURE_TEMPERATURE` | Air transport properties (`htc_extair`) | Tabulates viscosity, thermal conductivity, and $c_p$ over $(P, T)$. |
| `SATURATION_LINE` | Viscosity and conductivity at $Q = 0$ and $Q = 1$ | Saturation boundary transport properties. |

```bash
./run.sh                          # generate all grids into fixtures/auxtables
./run.sh /tmp/aux --only MEG      # generate single fluid family
./run.sh --sweep                  # error-vs-resolution ladder
```

---

## 1. Resolution & Accuracy Principles

- **Concentration Axis at 1% Steps**: Fluid property functions resolve integer mass percentages (e.g., `EG50` is `0.50`). Exact node placement eliminates interpolation error along the concentration axis.
- **Saturation Pressure Ceiling**: Saturation line calculations truncate at $0.75 \cdot p_{\text{crit}}$ where transport properties remain finite.
- **Non-Uniform Normalized Temperature Distribution**: Incompressible grids place temperature nodes according to error equidistribution of $\ln(\mu)$ to level piecewise-linear interpolation error across operating temperatures.

---

## 2. Binary Format (`FRAUX1`)

All integers and floating-point values are little-endian.

```text
Offset  Size  Field
  0       8   ASCII magic: "FRAUX1\0\0"
  8       2   u16 format_version = 1
 10       1   u8  elem_kind (0 = f64, 1 = f32)
 11       1   u8  flags (bit 0: ragged columns)
 12       4   u32 kind (0 = incompressible, 1 = pressure_temperature, 2 = saturation_line)
 16       4   u32 n1 (axis-1 samples)
 20       4   u32 n2 (axis-2 samples)
 24       4   u32 n_outputs
 28       4   u32 header_bytes (payload start offset, 8-byte aligned)
 32       8   f64 axis1_min
 40       8   f64 axis1_max
 48       8   f64 axis2_min
 56       8   f64 axis2_max
 64       8   f64 ref_pressure (for incompressible fluids)
 72       2   u16 name_len
 74       2   u16 version_len
 76       2   u16 axis1_name_len
 78       2   u16 axis2_name_len
 80       4   u32 reserved (0)
 84     ...   name, version, axis-1 name, axis-2 name (UTF-8)
        ...   per output: u16 name_len, name bytes, u8 transform (0 = linear, 1 = log)
        ...   zero padding to header_bytes

Payload:
      n1      axis1 coordinates
      n2      axis2 coordinates
   n1*n2      output plane per property (stored under corresponding transform)
```
