/**
 * User Slice - управление профилем пользователя
 */

import {createSlice, PayloadAction} from '@reduxjs/toolkit';

interface UserProfile {
  id: string;
  username: string;
  displayName: string;
  avatar: string | null;
  bio: string;
  publicKey: string;
}

interface UserState {
  profile: UserProfile | null;
  contacts: UserProfile[];
  loading: boolean;
  error: string | null;
}

const initialState: UserState = {
  profile: null,
  contacts: [],
  loading: false,
  error: null,
};

const userSlice = createSlice({
  name: 'user',
  initialState,
  reducers: {
    setProfile: (state, action: PayloadAction<UserProfile>) => {
      state.profile = action.payload;
    },
    updateProfile: (state, action: PayloadAction<Partial<UserProfile>>) => {
      if (state.profile) {
        state.profile = {...state.profile, ...action.payload};
      }
    },
    fetchContactsStart: (state) => {
      state.loading = true;
      state.error = null;
    },
    fetchContactsSuccess: (state, action: PayloadAction<UserProfile[]>) => {
      state.contacts = action.payload;
      state.loading = false;
    },
    fetchContactsFailure: (state, action: PayloadAction<string>) => {
      state.loading = false;
      state.error = action.payload;
    },
  },
});

export const {
  setProfile,
  updateProfile,
  fetchContactsStart,
  fetchContactsSuccess,
  fetchContactsFailure,
} = userSlice.actions;

export default userSlice.reducer;
