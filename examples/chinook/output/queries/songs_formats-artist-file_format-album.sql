-- name: songs-formats-artist-file-format-album
SELECT
    track.name AS title,
    artist.name AS artist_name,
    album.title AS album_name,
    media_type.name AS file_format
FROM
    album
    JOIN artist USING (artist_id)
    LEFT JOIN track USING (album_id)
    JOIN media_type USING (media_type_id)
WHERE
    artist.name = $1
    AND media_type.name = $2;

