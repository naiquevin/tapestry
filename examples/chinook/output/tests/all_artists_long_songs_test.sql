CREATE OR REPLACE FUNCTION all_artists_long_songs ()
    RETURNS SETOF record
    AS $$
    SELECT
        ar.artist_id,
        ar.name,
        max(milliseconds) * interval '1 ms' AS duration
    FROM
        track t
        INNER JOIN album al USING (album_id)
        INNER JOIN artist ar USING (artist_id)
        INNER JOIN genre g USING (genre_id)
    GROUP BY
        ar.artist_id
    ORDER BY
        -- Descending order because we want the top artists
        duration DESC
$$
LANGUAGE sql;

BEGIN;
SELECT
    plan (1);
-- start(noformat)
-- Run the tests.
SELECT is(count(*), 204::bigint) from all_artists_long_songs() AS (artist_id int, name text, duration interval);
-- Finish the tests and clean up.
-- end(noformat)
SELECT
    *
FROM
    finish ();
ROLLBACK;

