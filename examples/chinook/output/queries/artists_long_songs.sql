-- name: artists-long-songs
SELECT
    ar.artist_id,
    ar.name,
    max(milliseconds) * interval '1 ms' AS duration
FROM
    track t
    INNER JOIN album al USING (album_id)
    INNER JOIN artist ar USING (artist_id)
GROUP BY
    ar.artist_id
ORDER BY
    -- Descending order because we want the top artists
    duration DESC;

