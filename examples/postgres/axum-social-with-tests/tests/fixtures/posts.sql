INSERT INTO public.post (post_id, user_id, content, created_at)
VALUES
    -- from: alice
    ('d9ca2672-24c5-4442-b32f-cd717adffbaa', '51b374f1-93ae-4c5c-89dd-611bda8412ce',
     'This new computer is blazing fast!', '2022-07-29 01:36:24.679082'),
    -- from: bob
    ('7e3d4d16-a35e-46ba-8223-b4f1debbfbfe', 'c994b839-84f4-4509-ad49-59119133d6f5', '@alice is a haxxor',
     '2022-07-29 01:54:45.823523');
