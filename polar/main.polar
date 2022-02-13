# User rules

## all users can read other users
allow(_user: User, "read", _other_user: User);

## admins can update or delete other users
allow(user: User, action: String, _other_user: User) if
    user.role = "Administrator" and
    (action = "update" or action = "delete");

## moderators can update or delete other users with the exception of admins or other mods
allow(user: User, action: String, other_user: User) if
    (user.role = "Moderator" and
    (other_user.role != "Administrator" or other_user.role != "Moderator")) and
    (action = "update" or action = "delete");

## users can update or delete themselves
allow(user: User, action: String, other_user: User) if
    user.id = other_user.id and
    (action = "update" or action = "delete");
