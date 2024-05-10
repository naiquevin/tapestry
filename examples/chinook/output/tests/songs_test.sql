PREPARE song_formats (varchar, varchar) AS
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

BEGIN;
SELECT
    plan (1);
-- start(noformat)
-- Run the tests.
SELECT results_eq(
       'EXECUTE song_formats(''Iron Maiden'', ''Protected AAC audio file'')',
       $$VALUES
         ('Different World'::varchar, 'Iron Maiden'::varchar, 'A Matter of Life and Death'::varchar, 'Protected AAC audio file'::varchar),
         ('These Colours Don''t Run'::varchar, 'Iron Maiden'::varchar, 'A Matter of Life and Death'::varchar, 'Protected AAC audio file'::varchar),
         ('Brighter Than a Thousand Suns'::varchar, 'Iron Maiden'::varchar, 'A Matter of Life and Death'::varchar, 'Protected AAC audio file'::varchar),
         ('The Pilgrim'::varchar, 'Iron Maiden'::varchar, 'A Matter of Life and Death'::varchar, 'Protected AAC audio file'::varchar),
         ('The Longest Day'::varchar, 'Iron Maiden'::varchar, 'A Matter of Life and Death'::varchar, 'Protected AAC audio file'::varchar),
         ('Out of the Shadows'::varchar, 'Iron Maiden'::varchar, 'A Matter of Life and Death'::varchar, 'Protected AAC audio file'::varchar),
         ('The Reincarnation of Benjamin Breeg'::varchar, 'Iron Maiden'::varchar, 'A Matter of Life and Death'::varchar, 'Protected AAC audio file'::varchar),
         ('For the Greater Good of God'::varchar, 'Iron Maiden'::varchar, 'A Matter of Life and Death'::varchar, 'Protected AAC audio file'::varchar),
         ('Lord of Light'::varchar, 'Iron Maiden'::varchar, 'A Matter of Life and Death'::varchar, 'Protected AAC audio file'::varchar),
         ('The Legacy'::varchar, 'Iron Maiden'::varchar, 'A Matter of Life and Death'::varchar, 'Protected AAC audio file'::varchar),
         ('Hallowed Be Thy Name (Live) [Non Album Bonus Track]'::varchar, 'Iron Maiden'::varchar, 'A Matter of Life and Death'::varchar, 'Protected AAC audio file'::varchar)
       $$,
       'Verify return value'
);
-- end(noformat)
-- Finish the tests and clean up.
SELECT
    *
FROM
    finish ();
ROLLBACK;

