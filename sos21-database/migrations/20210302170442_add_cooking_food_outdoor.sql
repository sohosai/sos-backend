ALTER TYPE project_category ADD VALUE 'cooking';
ALTER TYPE project_category ADD VALUE 'food';

ALTER TABLE forms ALTER COLUMN unspecified_query
  SET DATA TYPE bit(16)
  USING unspecified_query || B'00000000';
ALTER TABLE forms ALTER COLUMN general_query
  SET DATA TYPE bit(16)
  USING general_query || B'00000000';
ALTER TABLE forms ALTER COLUMN stage_query
  SET DATA TYPE bit(16)
  USING stage_query || B'00000000';

ALTER TABLE forms ADD COLUMN food_query bit(16);
ALTER TABLE forms ADD COLUMN cooking_query bit(16);
UPDATE forms SET food_query = B'0000000000000000', cooking_query = B'0000000000000000';
ALTER TABLE forms ALTER COLUMN food_query SET NOT NULL;
ALTER TABLE forms ALTER COLUMN cooking_query SET NOT NULL;

ALTER TABLE forms ADD COLUMN needs_sync boolean;
UPDATE forms SET needs_sync = true;
ALTER TABLE forms ALTER COLUMN needs_sync SET NOT NULL;

DROP FUNCTION form_query_match(bit(8), integer);
CREATE FUNCTION form_query_match(bit(16), integer)
    RETURNS boolean
    IMMUTABLE LEAKPROOF
    LANGUAGE SQL AS $$
    SELECT $1 & (B'10000000000000000' >> $2::integer) <> B'0000000000000000';
$$;
