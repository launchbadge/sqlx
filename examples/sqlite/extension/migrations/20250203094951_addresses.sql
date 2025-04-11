create table addresses (address text, family integer);

-- The `ipfamily` function is provided by the
-- [ipaddr](https://github.com/nalgeon/sqlean/blob/main/docs/ipaddr.md)
-- sqlite extension, and so this migration can not run if that
-- extension is not loaded.
insert into addresses (address, family) values
  ('fd04:3d29:9f41::1', ipfamily('fd04:3d29:9f41::1')),
  ('10.0.0.1', ipfamily('10.0.0.1')),
  ('10.0.0.2', ipfamily('10.0.0.2')),
  ('fd04:3d29:9f41::2', ipfamily('fd04:3d29:9f41::2')),
  ('fd04:3d29:9f41::3', ipfamily('fd04:3d29:9f41::3')),
  ('10.0.0.3', ipfamily('10.0.0.3')),
  ('fd04:3d29:9f41::4', ipfamily('fd04:3d29:9f41::4')),
  ('fd04:3d29:9f41::5', ipfamily('fd04:3d29:9f41::5')),
  ('fd04:3d29:9f41::6', ipfamily('fd04:3d29:9f41::6')),
  ('10.0.0.4', ipfamily('10.0.0.4')),
  ('10.0.0.5', ipfamily('10.0.0.5')),
  ('10.0.0.6', ipfamily('10.0.0.6')),
  ('10.0.0.7', ipfamily('10.0.0.7')),
  ('fd04:3d29:9f41::7', ipfamily('fd04:3d29:9f41::7')),
  ('fd04:3d29:9f41::8', ipfamily('fd04:3d29:9f41::8')),
  ('10.0.0.8', ipfamily('10.0.0.8')),
  ('fd04:3d29:9f41::9', ipfamily('fd04:3d29:9f41::9')),
  ('10.0.0.9', ipfamily('10.0.0.9'));
