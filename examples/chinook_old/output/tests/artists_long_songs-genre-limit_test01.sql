PREPARE artists_long_songs (varchar, int) AS
SELECT
    ar.artist_id,
    ar.name,
    max(milliseconds) * interval '1 ms' AS duration
FROM
    track t
    INNER JOIN album al USING (album_id)
    INNER JOIN artist ar USING (artist_id)
    INNER JOIN genre g USING (genre_id)
WHERE
    g.name = $1
GROUP BY
    ar.artist_id
ORDER BY
    -- Descending order because we want the top artists
    duration DESC
LIMIT $2;

BEGIN;
SELECT
    plan (1);
-- start(noformat)
-- Run the tests.
SELECT results_eq(
    'EXECUTE artists_long_songs(''Rock'', 10)',
    $$VALUES
        (22, 'Led Zeppelin'::varchar, '00:26:52.329'::interval),
        (58, 'Deep Purple'::varchar, '00:19:56.094'::interval),
        (59, 'Santana'::varchar, '00:17:50.027'::interval),
        (136, 'Terry Bozzio, Tony Levin & Steve Stevens'::varchar, '00:14:40.64'::interval),
        (140, 'The Doors'::varchar, '00:11:41.831'::interval),
        (90, 'Iron Maiden'::varchar, '00:11:18.008'::interval),
        (23, 'Frank Zappa & Captain Beefheart'::varchar, '00:11:17.694'::interval),
        (128, 'Rush'::varchar, '00:11:07.428'::interval),
        (76, 'Creedence Clearwater Revival'::varchar, '00:11:04.894'::interval),
        (92, 'Jamiroquai'::varchar, '00:10:16.829'::interval)
    $$,
    'Verify return value'
);
-- Finish the tests and clean up.
-- end(noformat)
SELECT
    *
FROM
    finish ();
ROLLBACK;

